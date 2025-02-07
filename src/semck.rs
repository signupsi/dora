use crate::class::TypeParams;
use crate::mem;
use crate::sym::Sym::{SymClass, SymClassTypeParam, SymFctTypeParam, SymStruct, SymTrait};
use crate::ty::BuiltinType;
use crate::typeck;
use crate::vm::{NodeMap, VM};
use dora_parser::ast::Type::{TypeBasic, TypeLambda, TypeSelf, TypeTuple};
use dora_parser::ast::{Stmt, Type};
use dora_parser::error::msg::Msg;

mod abstractck;
mod clsdefck;
mod constdefck;
mod fctdefck;
mod flowck;
mod globaldef;
mod globaldefck;
mod implck;
mod impldefck;
mod nameck;
mod prelude;
mod returnck;
pub mod specialize;
mod structdefck;
mod superck;
mod traitdefck;

macro_rules! return_on_error {
    ($vm: ident) => {{
        if $vm.diag.lock().has_errors() {
            return;
        }
    }};
}

pub fn check<'ast>(vm: &mut VM<'ast>) {
    let mut map_cls_defs = NodeMap::new(); // get ClassId from ast node
    let mut map_struct_defs = NodeMap::new(); // get StructId from ast node
    let mut map_trait_defs = NodeMap::new(); // get TraitId from ast node
    let mut map_impl_defs = NodeMap::new(); // get ImplId from ast node
    let mut map_global_defs = NodeMap::new(); // get GlobalId from ast node
    let mut map_const_defs = NodeMap::new(); // get ConstId from ast node

    // add user defined fcts and classes to vm
    // this check does not look into fct or class bodies
    globaldef::check(
        vm,
        &mut map_cls_defs,
        &mut map_struct_defs,
        &mut map_trait_defs,
        &mut map_impl_defs,
        &mut map_global_defs,
        &mut map_const_defs,
    );
    return_on_error!(vm);

    // define internal classes
    prelude::internal_classes(vm);

    // checks class/struct/trait definitions/bodies
    clsdefck::check(vm, &vm.ast, &map_cls_defs);
    structdefck::check(vm, &vm.ast, &map_struct_defs);
    traitdefck::check(vm, &vm.ast, &map_trait_defs);
    impldefck::check(vm, &vm.ast, &map_impl_defs);
    globaldefck::check(vm, &vm.ast, &map_global_defs);
    constdefck::check(vm, &vm.ast, &map_const_defs);
    return_on_error!(vm);

    // check names/identifiers of local variables
    // and their usage (variable def/use, function calls) in function bodies
    nameck::check(vm);
    return_on_error!(vm);

    // check type definitions of params,
    // return types and local variables in functions
    fctdefck::check(vm);
    return_on_error!(vm);

    superck::check_override(vm);
    return_on_error!(vm);

    // check impl methods against trait definition
    implck::check(vm);
    return_on_error!(vm);

    // define internal functions
    prelude::internal_functions(vm);

    // check types of expressions in functions
    typeck::check(vm);
    return_on_error!(vm);

    // are break and continue used in the right places?
    flowck::check(vm);

    // checks if function has a return value
    returnck::check(vm);

    // add size of super classes to field offsets
    superck::check(vm);
    return_on_error!(vm);

    abstractck::check(vm);

    // check for internal functions or classes
    internalck(vm);
    return_on_error!(vm);

    // initialize addresses for global variables
    init_global_addresses(vm);
}

fn internalck<'ast>(vm: &VM<'ast>) {
    for fct in vm.fcts.iter() {
        let fct = fct.read();

        if fct.in_class() {
            continue;
        }

        if fct.internal && !fct.internal_resolved {
            vm.diag
                .lock()
                .report_without_path(fct.pos, Msg::UnresolvedInternal);
        }

        if fct.kind.is_definition() && !fct.in_trait() {
            vm.diag
                .lock()
                .report_without_path(fct.pos, Msg::MissingFctBody);
        }
    }

    for cls in vm.classes.iter() {
        let cls = cls.read();

        if cls.internal && !cls.internal_resolved {
            vm.diag
                .lock()
                .report_without_path(cls.pos, Msg::UnresolvedInternal);
        }

        for method in &cls.methods {
            let method = vm.fcts.idx(*method);
            let method = method.read();

            if method.internal && !method.internal_resolved {
                vm.diag
                    .lock()
                    .report_without_path(method.pos, Msg::UnresolvedInternal);
            }

            if method.kind.is_definition() && !method.is_abstract {
                vm.diag
                    .lock()
                    .report_without_path(method.pos, Msg::MissingFctBody);
            }
        }
    }
}

fn init_global_addresses<'ast>(vm: &VM<'ast>) {
    let globals = vm.globals.lock();
    let mut size = 0;
    let mut offsets = Vec::with_capacity(globals.len());

    for glob in globals.iter() {
        let glob = glob.lock();

        let ty_size = glob.ty.size(vm);
        let ty_align = glob.ty.align(vm);

        let offset = mem::align_i32(size, ty_align);
        offsets.push(offset);
        size = offset + ty_size;
    }

    let ptr = vm.gc.alloc_perm(size as usize);

    for (ind, glob) in globals.iter().enumerate() {
        let mut glob = glob.lock();
        let offset = offsets[ind];

        glob.address_value = ptr.offset(offset as usize);
    }
}

pub fn read_type<'ast>(vm: &VM<'ast>, t: &'ast Type) -> Option<BuiltinType> {
    match *t {
        TypeSelf(_) => {
            return Some(BuiltinType::This);
        }

        TypeBasic(ref basic) => {
            let sym = vm.sym.lock().get(basic.name);
            if let Some(sym) = sym {
                match sym {
                    SymClass(cls_id) => {
                        let ty = if basic.params.len() > 0 {
                            let mut type_params = Vec::new();

                            for param in &basic.params {
                                let param = read_type(vm, param);

                                if let Some(param) = param {
                                    type_params.push(param);
                                } else {
                                    return None;
                                }
                            }

                            let cls = vm.classes.idx(cls_id);
                            let cls = cls.read();

                            if cls.type_params.len() != type_params.len() {
                                let msg = Msg::WrongNumberTypeParams(
                                    cls.type_params.len(),
                                    type_params.len(),
                                );
                                vm.diag.lock().report_without_path(basic.pos, msg);
                                return None;
                            }

                            for (tp, ty) in cls.type_params.iter().zip(type_params.iter()) {
                                if let Some(cls_id) = tp.class_bound {
                                    let cls = vm.cls(cls_id);

                                    if !ty.subclass_from(vm, cls) {
                                        let name = ty.name(vm);
                                        let cls = cls.name(vm);

                                        let msg = Msg::ClassBoundNotSatisfied(name, cls);
                                        vm.diag.lock().report_without_path(basic.pos, msg);
                                    }
                                }

                                let cls_id = if let Some(cls_id) = ty.cls_id(vm) {
                                    cls_id
                                } else {
                                    continue;
                                };

                                let cls = vm.classes.idx(cls_id);
                                let cls = cls.read();

                                for &trait_bound in &tp.trait_bounds {
                                    if !cls.traits.contains(&trait_bound) {
                                        let bound = vm.traits[trait_bound].read();
                                        let name = ty.name(vm);
                                        let trait_name = vm.interner.str(bound.name).to_string();
                                        let msg = Msg::TraitBoundNotSatisfied(name, trait_name);
                                        vm.diag.lock().report_without_path(bound.pos, msg);
                                    }
                                }
                            }

                            let list_id = vm.lists.lock().insert(type_params.into());
                            BuiltinType::Class(cls.id, list_id)
                        } else {
                            let cls = vm.classes.idx(cls_id);
                            let cls = cls.read();

                            cls.ty
                        };

                        return Some(ty);
                    }

                    SymTrait(trait_id) => {
                        if basic.params.len() > 0 {
                            let msg = Msg::NoTypeParamsExpected;
                            vm.diag.lock().report_without_path(basic.pos, msg);
                        }

                        return Some(BuiltinType::Trait(trait_id));
                    }

                    SymStruct(struct_id) => {
                        if basic.params.len() > 0 {
                            let msg = Msg::NoTypeParamsExpected;
                            vm.diag.lock().report_without_path(basic.pos, msg);
                        }

                        let list_id = vm.lists.lock().insert(TypeParams::empty());
                        return Some(BuiltinType::Struct(struct_id, list_id));
                    }

                    SymClassTypeParam(cls_id, type_param_id) => {
                        if basic.params.len() > 0 {
                            let msg = Msg::NoTypeParamsExpected;
                            vm.diag.lock().report_without_path(basic.pos, msg);
                        }

                        return Some(BuiltinType::ClassTypeParam(cls_id, type_param_id));
                    }

                    SymFctTypeParam(fct_id, type_param_id) => {
                        if basic.params.len() > 0 {
                            let msg = Msg::NoTypeParamsExpected;
                            vm.diag.lock().report_without_path(basic.pos, msg);
                        }

                        return Some(BuiltinType::FctTypeParam(fct_id, type_param_id));
                    }

                    _ => {
                        let name = vm.interner.str(basic.name).to_string();
                        let msg = Msg::ExpectedType(name);
                        vm.diag.lock().report_without_path(basic.pos, msg);
                    }
                }
            } else {
                let name = vm.interner.str(basic.name).to_string();
                let msg = Msg::UnknownType(name);
                vm.diag.lock().report_without_path(basic.pos, msg);
            }
        }

        TypeTuple(ref tuple) if tuple.subtypes.len() == 0 => {
            return Some(BuiltinType::Unit);
        }

        TypeLambda(ref lambda) => {
            let mut params = vec![];

            for param in &lambda.params {
                if let Some(p) = read_type(vm, param) {
                    params.push(p);
                } else {
                    return None;
                }
            }

            let ret = if let Some(ret) = read_type(vm, &lambda.ret) {
                ret
            } else {
                return None;
            };

            let ty = vm.lambda_types.lock().insert(params, ret);
            let ty = BuiltinType::Lambda(ty);

            return Some(ty);
        }

        _ => vm
            .diag
            .lock()
            .report_unimplemented("unknown file".to_string(), t.pos()),
    }

    None
}

pub fn always_returns(s: &Stmt) -> bool {
    match returnck::returns_value(s) {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[cfg(test)]
pub mod tests {
    use crate::test;
    use crate::vm::VM;
    use dora_parser::error::msg::Msg;
    use dora_parser::lexer::position::Position;

    pub fn ok(code: &'static str) {
        test::parse_with_errors(code, |vm| {
            let diag = vm.diag.lock();
            let errors = diag.errors();

            println!("errors = {:?}", errors);

            for e in errors {
                println!("{}", e.message());
            }

            assert!(!diag.has_errors());
        });
    }

    pub fn ok_with_test<F, R>(code: &'static str, f: F) -> R
    where
        F: FnOnce(&VM) -> R,
    {
        test::parse_with_errors(code, |vm| {
            let diag = vm.diag.lock();
            let errors = diag.errors();

            println!("errors = {:?}", errors);

            for e in errors {
                println!("{}", e.message());
            }

            assert!(!diag.has_errors());

            f(vm)
        })
    }

    pub fn err(code: &'static str, pos: Position, msg: Msg) {
        test::parse_with_errors(code, |vm| {
            let diag = vm.diag.lock();
            let errors = diag.errors();

            println!("errors = {:?}", errors);

            assert_eq!(1, errors.len(), "found {} errors instead", errors.len());
            assert_eq!(pos, errors[0].pos);
            assert_eq!(msg, errors[0].msg);
        });
    }

    pub fn errors(code: &'static str, vec: &[(Position, Msg)]) {
        test::parse_with_errors(code, |vm| {
            let diag = vm.diag.lock();
            let errors = diag.errors();

            println!("errors = {:?}", errors);
            assert_eq!(vec.len(), errors.len());

            for (ind, error) in errors.iter().enumerate() {
                assert_eq!(vec[ind].0, error.pos);
                assert_eq!(vec[ind].1, error.msg);
            }
        });
    }

    pub fn pos(line: u32, col: u32) -> Position {
        Position::new(line, col)
    }
}
