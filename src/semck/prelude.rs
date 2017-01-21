use class::ClassId;
use ctxt::{Context, FctKind, Intrinsic};
use stdlib;
use ty::BuiltinType;

pub fn internal_classes<'ast>(ctxt: &mut Context<'ast>) {
    let size = BuiltinType::Byte.size(ctxt);
    ctxt.primitive_classes.byte_class = internal_class(ctxt, "byte", BuiltinType::Byte, size);

    let size = BuiltinType::Int.size(ctxt);
    ctxt.primitive_classes.int_class = internal_class(ctxt, "int", BuiltinType::Int, size);

    let size = BuiltinType::Long.size(ctxt);
    ctxt.primitive_classes.long_class = internal_class(ctxt, "long", BuiltinType::Long, size);

    let size = BuiltinType::Bool.size(ctxt);
    ctxt.primitive_classes.bool_class = internal_class(ctxt, "bool", BuiltinType::Bool, size);

    ctxt.primitive_classes.str_class = internal_class(ctxt, "Str", BuiltinType::Str, 0);
    ctxt.primitive_classes.int_array = internal_class(ctxt, "IntArray", BuiltinType::IntArray, 0);
}

fn internal_class<'ast>(ctxt: &mut Context<'ast>,
                        name: &str,
                        ty: BuiltinType,
                        size: i32)
                        -> ClassId {
    let iname = ctxt.interner.intern(name);
    let clsid = ctxt.sym.borrow().get_class(iname);

    if let Some(clsid) = clsid {
        let mut cls = ctxt.classes[clsid].borrow_mut();

        if cls.internal {
            cls.ty = ty;
            cls.size = size;
            cls.internal = false;
        }

        clsid
    } else {
        panic!("class {} not found!", name);
    }
}

pub fn internal_functions<'ast>(ctxt: &mut Context<'ast>) {
    native_fct(ctxt, "print", stdlib::print as *const u8);
    native_fct(ctxt, "println", stdlib::println as *const u8);
    intrinsic_fct(ctxt, "assert", Intrinsic::Assert);
    native_fct(ctxt, "argc", stdlib::argc as *const u8);
    native_fct(ctxt, "argv", stdlib::argv as *const u8);
    native_fct(ctxt, "forceCollect", stdlib::gc_collect as *const u8);
    native_fct(ctxt,
               "intArrayWith",
               stdlib::ctor_int_array_elem as *const u8);
    native_fct(ctxt,
               "emptyIntArray",
               stdlib::ctor_int_array_empty as *const u8);

    let clsid = ctxt.primitive_classes.byte_class;
    native_method(ctxt, clsid, "toString", stdlib::byte_to_string as *const u8);

    let clsid = ctxt.primitive_classes.int_class;
    native_method(ctxt, clsid, "toString", stdlib::int_to_string as *const u8);

    let clsid = ctxt.primitive_classes.long_class;
    native_method(ctxt, clsid, "toString", stdlib::long_to_string as *const u8);

    let clsid = ctxt.primitive_classes.bool_class;
    native_method(ctxt, clsid, "toInt", stdlib::bool_to_int as *const u8);
    native_method(ctxt, clsid, "toString", stdlib::bool_to_string as *const u8);

    let clsid = ctxt.primitive_classes.str_class;
    native_method(ctxt, clsid, "len", stdlib::str_len as *const u8);
    native_method(ctxt, clsid, "parseInt", stdlib::str_parse_int as *const u8);

    let clsid = ctxt.primitive_classes.int_array;
    intrinsic_method(ctxt, clsid, "len", Intrinsic::IntArrayLen);
    intrinsic_method(ctxt, clsid, "get", Intrinsic::IntArrayGet);
    intrinsic_method(ctxt, clsid, "set", Intrinsic::IntArraySet);
}

fn native_method<'ast>(ctxt: &mut Context<'ast>, clsid: ClassId, name: &str, fctptr: *const u8) {
    internal_method(ctxt, clsid, name, FctKind::Native(fctptr));
}

fn intrinsic_method<'ast>(ctxt: &mut Context<'ast>,
                          clsid: ClassId,
                          name: &str,
                          intrinsic: Intrinsic) {
    internal_method(ctxt, clsid, name, FctKind::Builtin(intrinsic));
}

fn internal_method<'ast>(ctxt: &mut Context<'ast>,
                         clsid: ClassId,
                         name: &str,
                         kind: FctKind<'ast>) {
    let cls = ctxt.classes[clsid].borrow();
    let name = ctxt.interner.intern(name);

    for &mid in &cls.methods {
        let mut mtd = ctxt.fcts[mid].borrow_mut();

        if mtd.name == name && mtd.internal {
            mtd.kind = kind;
            mtd.internal = false;
            break;
        }
    }
}

fn native_fct<'ast>(ctxt: &mut Context<'ast>, name: &str, fctptr: *const u8) {
    internal_fct(ctxt, name, FctKind::Native(fctptr));
}

fn intrinsic_fct<'ast>(ctxt: &mut Context<'ast>, name: &str, intrinsic: Intrinsic) {
    internal_fct(ctxt, name, FctKind::Builtin(intrinsic));
}

fn internal_fct<'ast>(ctxt: &mut Context<'ast>, name: &str, kind: FctKind<'ast>) {
    let name = ctxt.interner.intern(name);
    let fctid = ctxt.sym.borrow().get_fct(name);

    if let Some(fctid) = fctid {
        let mut fct = ctxt.fcts[fctid].borrow_mut();

        if fct.internal {
            fct.kind = kind;
            fct.internal = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use semck::tests::*;

    #[test]
    fn builtin_functions() {
        ok("fun f() { assert(true); }");
        ok("fun f() { print(\"test\"); }");
        ok("fun f() { println(\"test\"); }");
    }
}
