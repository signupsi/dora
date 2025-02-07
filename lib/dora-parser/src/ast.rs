use std::fmt;
use std::slice::Iter;

use crate::ast::Elem::*;
use crate::interner::{Interner, Name};
use crate::lexer::position::{Position, Span};
use crate::lexer::token::{FloatSuffix, IntBase, IntSuffix};

pub mod dump;
pub mod visit;

#[derive(Clone, Debug)]
pub struct Ast {
    pub files: Vec<File>,
}

impl Ast {
    pub fn new() -> Ast {
        Ast { files: Vec::new() }
    }

    #[cfg(test)]
    pub fn fct0(&self) -> &Function {
        self.files.last().unwrap().elements[0]
            .to_function()
            .unwrap()
    }

    #[cfg(test)]
    pub fn fct(&self, index: usize) -> &Function {
        self.files.last().unwrap().elements[index]
            .to_function()
            .unwrap()
    }

    #[cfg(test)]
    pub fn cls0(&self) -> &Class {
        self.files.last().unwrap().elements[0].to_class().unwrap()
    }

    #[cfg(test)]
    pub fn cls(&self, index: usize) -> &Class {
        self.files.last().unwrap().elements[index]
            .to_class()
            .unwrap()
    }

    #[cfg(test)]
    pub fn struct0(&self) -> &Struct {
        self.files.last().unwrap().elements[0].to_struct().unwrap()
    }

    #[cfg(test)]
    pub fn trai(&self, index: usize) -> &Trait {
        self.files.last().unwrap().elements[index]
            .to_trait()
            .unwrap()
    }

    #[cfg(test)]
    pub fn trait0(&self) -> &Trait {
        self.files.last().unwrap().elements[0].to_trait().unwrap()
    }

    #[cfg(test)]
    pub fn impl0(&self) -> &Impl {
        self.files.last().unwrap().elements[0].to_impl().unwrap()
    }

    #[cfg(test)]
    pub fn global0(&self) -> &Global {
        self.files.last().unwrap().elements[0].to_global().unwrap()
    }

    #[cfg(test)]
    pub fn const0(&self) -> &Const {
        self.files.last().unwrap().elements[0].to_const().unwrap()
    }
}

#[derive(Clone, Debug)]
pub struct File {
    pub path: String,
    pub elements: Vec<Elem>,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
pub struct NodeId(pub usize);

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

#[derive(Clone, Debug)]
pub enum Elem {
    ElemFunction(Function),
    ElemClass(Class),
    ElemStruct(Struct),
    ElemTrait(Trait),
    ElemImpl(Impl),
    ElemGlobal(Global),
    ElemConst(Const),
}

impl Elem {
    pub fn id(&self) -> NodeId {
        match self {
            &ElemFunction(ref fct) => fct.id,
            &ElemClass(ref class) => class.id,
            &ElemStruct(ref s) => s.id,
            &ElemTrait(ref t) => t.id,
            &ElemImpl(ref i) => i.id,
            &ElemGlobal(ref g) => g.id,
            &ElemConst(ref c) => c.id,
        }
    }

    pub fn to_function(&self) -> Option<&Function> {
        match self {
            &ElemFunction(ref fct) => Some(fct),
            _ => None,
        }
    }

    pub fn to_class(&self) -> Option<&Class> {
        match self {
            &ElemClass(ref class) => Some(class),
            _ => None,
        }
    }

    pub fn to_struct(&self) -> Option<&Struct> {
        match self {
            &ElemStruct(ref struc) => Some(struc),
            _ => None,
        }
    }

    pub fn to_trait(&self) -> Option<&Trait> {
        match self {
            &ElemTrait(ref trai) => Some(trai),
            _ => None,
        }
    }

    pub fn to_impl(&self) -> Option<&Impl> {
        match self {
            &ElemImpl(ref ximpl) => Some(ximpl),
            _ => None,
        }
    }

    pub fn to_global(&self) -> Option<&Global> {
        match self {
            &ElemGlobal(ref global) => Some(global),
            _ => None,
        }
    }

    pub fn to_const(&self) -> Option<&Const> {
        match self {
            &ElemConst(ref konst) => Some(konst),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Global {
    pub id: NodeId,
    pub pos: Position,
    pub name: Name,
    pub reassignable: bool,
    pub data_type: Type,
    pub expr: Option<Box<Expr>>,
}

#[derive(Clone, Debug)]
pub struct Const {
    pub id: NodeId,
    pub pos: Position,
    pub name: Name,
    pub data_type: Type,
    pub expr: Box<Expr>,
}

#[derive(Clone, Debug)]
pub struct Struct {
    pub id: NodeId,
    pub pos: Position,
    pub name: Name,
    pub fields: Vec<StructField>,
}

#[derive(Clone, Debug)]
pub struct StructField {
    pub id: NodeId,
    pub name: Name,
    pub pos: Position,
    pub data_type: Type,
}

#[derive(Clone, Debug)]
pub enum Type {
    TypeSelf(TypeSelfType),
    TypeBasic(TypeBasicType),
    TypeTuple(TypeTupleType),
    TypeLambda(TypeLambdaType),
}

#[derive(Clone, Debug)]
pub struct TypeSelfType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct TypeTupleType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub subtypes: Vec<Box<Type>>,
}

#[derive(Clone, Debug)]
pub struct TypeLambdaType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub params: Vec<Box<Type>>,
    pub ret: Box<Type>,
}

#[derive(Clone, Debug)]
pub struct TypeBasicType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub name: Name,
    pub params: Vec<Box<Type>>,
}

impl Type {
    pub fn create_self(id: NodeId, pos: Position, span: Span) -> Type {
        Type::TypeSelf(TypeSelfType {
            id: id,
            pos: pos,
            span: span,
        })
    }

    pub fn create_basic(
        id: NodeId,
        pos: Position,
        span: Span,
        name: Name,
        params: Vec<Box<Type>>,
    ) -> Type {
        Type::TypeBasic(TypeBasicType {
            id: id,
            pos: pos,
            span: span,
            name: name,
            params: params,
        })
    }

    pub fn create_fct(
        id: NodeId,
        pos: Position,
        span: Span,
        params: Vec<Box<Type>>,
        ret: Box<Type>,
    ) -> Type {
        Type::TypeLambda(TypeLambdaType {
            id: id,
            pos: pos,
            span: span,
            params: params,
            ret: ret,
        })
    }

    pub fn create_tuple(id: NodeId, pos: Position, span: Span, subtypes: Vec<Box<Type>>) -> Type {
        Type::TypeTuple(TypeTupleType {
            id: id,
            pos: pos,
            span: span,
            subtypes: subtypes,
        })
    }

    pub fn to_basic(&self) -> Option<&TypeBasicType> {
        match *self {
            Type::TypeBasic(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn to_basic_without_type_params(&self) -> Option<Name> {
        match *self {
            Type::TypeBasic(ref basic) => {
                if basic.params.len() == 0 {
                    Some(basic.name)
                } else {
                    None
                }
            }

            _ => None,
        }
    }

    pub fn to_tuple(&self) -> Option<&TypeTupleType> {
        match *self {
            Type::TypeTuple(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn to_fct(&self) -> Option<&TypeLambdaType> {
        match *self {
            Type::TypeLambda(ref val) => Some(val),
            _ => None,
        }
    }

    #[cfg(test)]
    pub fn is_unit(&self) -> bool {
        match self {
            &Type::TypeTuple(ref val) if val.subtypes.len() == 0 => true,
            _ => false,
        }
    }

    pub fn to_string(&self, interner: &Interner) -> String {
        match *self {
            Type::TypeSelf(_) => "Self".into(),
            Type::TypeBasic(ref val) => format!("{}", *interner.str(val.name)),

            Type::TypeTuple(ref val) => {
                let types: Vec<String> =
                    val.subtypes.iter().map(|t| t.to_string(interner)).collect();

                format!("({})", types.join(", "))
            }

            Type::TypeLambda(ref val) => {
                let types: Vec<String> = val.params.iter().map(|t| t.to_string(interner)).collect();
                let ret = val.ret.to_string(interner);

                format!("({}) -> {}", types.join(", "), ret)
            }
        }
    }

    pub fn pos(&self) -> Position {
        match *self {
            Type::TypeSelf(ref val) => val.pos,
            Type::TypeBasic(ref val) => val.pos,
            Type::TypeTuple(ref val) => val.pos,
            Type::TypeLambda(ref val) => val.pos,
        }
    }

    pub fn id(&self) -> NodeId {
        match *self {
            Type::TypeSelf(ref val) => val.id,
            Type::TypeBasic(ref val) => val.id,
            Type::TypeTuple(ref val) => val.id,
            Type::TypeLambda(ref val) => val.id,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Impl {
    pub id: NodeId,
    pub pos: Position,

    pub type_params: Option<Vec<TypeParam>>,
    pub trait_type: Option<Type>,
    pub class_type: Type,
    pub methods: Vec<Function>,
}

#[derive(Clone, Debug)]
pub struct Trait {
    pub id: NodeId,
    pub name: Name,
    pub pos: Position,
    pub methods: Vec<Function>,
}

#[derive(Clone, Debug)]
pub struct Class {
    pub id: NodeId,
    pub name: Name,
    pub pos: Position,
    pub parent_class: Option<ParentClass>,
    pub has_open: bool,
    pub is_abstract: bool,
    pub internal: bool,
    pub has_constructor: bool,

    pub constructor: Option<Function>,
    pub fields: Vec<Field>,
    pub methods: Vec<Function>,
    pub initializers: Vec<Box<Stmt>>,
    pub type_params: Option<Vec<TypeParam>>,
}

#[derive(Clone, Debug)]
pub struct TypeParam {
    pub name: Name,
    pub pos: Position,
    pub bounds: Vec<Type>,
}

#[derive(Clone, Debug)]
pub struct ConstructorParam {
    pub name: Name,
    pub pos: Position,
    pub data_type: Type,
    pub field: bool,
    pub reassignable: bool,
}

#[derive(Clone, Debug)]
pub struct ParentClass {
    pub name: Name,
    pub pos: Position,
    pub type_params: Option<Vec<TypeParam>>,
    pub params: Vec<Box<Expr>>,
}

impl ParentClass {
    pub fn new(
        name: Name,
        pos: Position,
        type_params: Option<Vec<TypeParam>>,
        params: Vec<Box<Expr>>,
    ) -> ParentClass {
        ParentClass {
            name: name,
            pos: pos,
            type_params: type_params,
            params: params,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Field {
    pub id: NodeId,
    pub name: Name,
    pub pos: Position,
    pub data_type: Type,
    pub primary_ctor: bool,
    pub expr: Option<Box<Expr>>,
    pub reassignable: bool,
}

#[derive(Clone, Debug)]
pub struct Function {
    pub id: NodeId,
    pub name: Name,
    pub pos: Position,
    pub method: bool,
    pub has_open: bool,
    pub has_override: bool,
    pub has_final: bool,
    pub has_optimize: bool,
    pub is_pub: bool,
    pub is_static: bool,
    pub is_abstract: bool,
    pub internal: bool,
    pub is_constructor: bool,

    pub params: Vec<Param>,
    pub throws: bool,

    pub return_type: Option<Type>,
    pub block: Option<Box<Stmt>>,
    pub type_params: Option<Vec<TypeParam>>,
}

impl Function {
    pub fn block(&self) -> &Stmt {
        self.block.as_ref().unwrap()
    }
}

#[derive(Clone, Debug)]
pub struct Modifiers(Vec<ModifierElement>);

impl Modifiers {
    pub fn new() -> Modifiers {
        Modifiers(Vec::new())
    }

    pub fn contains(&self, modifier: Modifier) -> bool {
        self.0.iter().find(|el| el.value == modifier).is_some()
    }

    pub fn add(&mut self, modifier: Modifier, pos: Position) {
        self.0.push(ModifierElement {
            value: modifier,
            pos: pos,
        });
    }

    pub fn iter(&self) -> Iter<ModifierElement> {
        self.0.iter()
    }
}

#[derive(Clone, Debug)]
pub struct ModifierElement {
    pub value: Modifier,
    pub pos: Position,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Modifier {
    Abstract,
    Override,
    Open,
    Final,
    Internal,
    Optimize,
    Pub,
    Static,
}

impl Modifier {
    pub fn name(&self) -> &'static str {
        match *self {
            Modifier::Abstract => "abstract",
            Modifier::Open => "open",
            Modifier::Override => "override",
            Modifier::Final => "final",
            Modifier::Internal => "internal",
            Modifier::Optimize => "optimize",
            Modifier::Pub => "pub",
            Modifier::Static => "static",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Param {
    pub id: NodeId,
    pub idx: u32,
    pub reassignable: bool,
    pub name: Name,
    pub pos: Position,
    pub data_type: Type,
}

#[derive(Clone, Debug)]
pub enum Stmt {
    StmtVar(StmtVarType),
    StmtWhile(StmtWhileType),
    StmtLoop(StmtLoopType),
    StmtIf(StmtIfType),
    StmtExpr(StmtExprType),
    StmtBlock(StmtBlockType),
    StmtBreak(StmtBreakType),
    StmtContinue(StmtContinueType),
    StmtReturn(StmtReturnType),
    StmtThrow(StmtThrowType),
    StmtDefer(StmtDeferType),
    StmtDo(StmtDoType),
    StmtSpawn(StmtSpawnType),
    StmtFor(StmtForType),
}

impl Stmt {
    pub fn create_var(
        id: NodeId,
        pos: Position,
        name: Name,
        reassignable: bool,
        data_type: Option<Type>,
        expr: Option<Box<Expr>>,
    ) -> Stmt {
        Stmt::StmtVar(StmtVarType {
            id: id,
            pos: pos,
            span: Span::invalid(),

            name: name,
            reassignable: reassignable,
            data_type: data_type,
            expr: expr,
        })
    }

    pub fn create_for(
        id: NodeId,
        pos: Position,
        name: Name,
        expr: Box<Expr>,
        block: Box<Stmt>,
    ) -> Stmt {
        Stmt::StmtFor(StmtForType {
            id: id,
            pos: pos,
            span: Span::invalid(),

            name: name,
            expr: expr,
            block: block,
        })
    }

    pub fn create_while(id: NodeId, pos: Position, cond: Box<Expr>, block: Box<Stmt>) -> Stmt {
        Stmt::StmtWhile(StmtWhileType {
            id: id,
            pos: pos,
            span: Span::invalid(),

            cond: cond,
            block: block,
        })
    }

    pub fn create_loop(id: NodeId, pos: Position, block: Box<Stmt>) -> Stmt {
        Stmt::StmtLoop(StmtLoopType {
            id: id,
            pos: pos,
            span: Span::invalid(),

            block: block,
        })
    }

    pub fn create_if(
        id: NodeId,
        pos: Position,
        cond: Box<Expr>,
        then_block: Box<Stmt>,
        else_block: Option<Box<Stmt>>,
    ) -> Stmt {
        Stmt::StmtIf(StmtIfType {
            id: id,
            pos: pos,
            span: Span::invalid(),

            cond: cond,
            then_block: then_block,
            else_block: else_block,
        })
    }

    pub fn create_expr(id: NodeId, pos: Position, expr: Box<Expr>) -> Stmt {
        Stmt::StmtExpr(StmtExprType {
            id: id,
            pos: pos,
            span: Span::invalid(),

            expr: expr,
        })
    }

    pub fn create_block(id: NodeId, pos: Position, stmts: Vec<Box<Stmt>>) -> Stmt {
        Stmt::StmtBlock(StmtBlockType {
            id: id,
            pos: pos,
            span: Span::invalid(),

            stmts: stmts,
        })
    }

    pub fn create_break(id: NodeId, pos: Position) -> Stmt {
        Stmt::StmtBreak(StmtBreakType {
            id: id,
            pos: pos,
            span: Span::invalid(),
        })
    }

    pub fn create_continue(id: NodeId, pos: Position) -> Stmt {
        Stmt::StmtContinue(StmtContinueType {
            id: id,
            pos: pos,
            span: Span::invalid(),
        })
    }

    pub fn create_return(id: NodeId, pos: Position, expr: Option<Box<Expr>>) -> Stmt {
        Stmt::StmtReturn(StmtReturnType {
            id: id,
            pos: pos,
            span: Span::invalid(),

            expr: expr,
        })
    }

    pub fn create_throw(id: NodeId, pos: Position, expr: Box<Expr>) -> Stmt {
        Stmt::StmtThrow(StmtThrowType {
            id: id,
            pos: pos,
            span: Span::invalid(),

            expr: expr,
        })
    }

    pub fn create_defer(id: NodeId, pos: Position, expr: Box<Expr>) -> Stmt {
        Stmt::StmtDefer(StmtDeferType {
            id: id,
            pos: pos,
            span: Span::invalid(),

            expr: expr,
        })
    }

    pub fn create_do(
        id: NodeId,
        pos: Position,
        do_block: Box<Stmt>,
        catch_blocks: Vec<CatchBlock>,
        finally_block: Option<FinallyBlock>,
    ) -> Stmt {
        Stmt::StmtDo(StmtDoType {
            id: id,
            pos: pos,
            span: Span::invalid(),

            do_block: do_block,
            catch_blocks: catch_blocks,
            finally_block: finally_block,
        })
    }

    pub fn create_spawn(id: NodeId, pos: Position, expr: Box<Expr>) -> Stmt {
        Stmt::StmtSpawn(StmtSpawnType {
            id: id,
            pos: pos,
            span: Span::invalid(),

            expr: expr,
        })
    }

    pub fn id(&self) -> NodeId {
        match *self {
            Stmt::StmtVar(ref stmt) => stmt.id,
            Stmt::StmtWhile(ref stmt) => stmt.id,
            Stmt::StmtFor(ref stmt) => stmt.id,
            Stmt::StmtLoop(ref stmt) => stmt.id,
            Stmt::StmtIf(ref stmt) => stmt.id,
            Stmt::StmtExpr(ref stmt) => stmt.id,
            Stmt::StmtBlock(ref stmt) => stmt.id,
            Stmt::StmtBreak(ref stmt) => stmt.id,
            Stmt::StmtContinue(ref stmt) => stmt.id,
            Stmt::StmtReturn(ref stmt) => stmt.id,
            Stmt::StmtThrow(ref stmt) => stmt.id,
            Stmt::StmtDefer(ref stmt) => stmt.id,
            Stmt::StmtDo(ref stmt) => stmt.id,
            Stmt::StmtSpawn(ref stmt) => stmt.id,
        }
    }

    pub fn pos(&self) -> Position {
        match *self {
            Stmt::StmtVar(ref stmt) => stmt.pos,
            Stmt::StmtWhile(ref stmt) => stmt.pos,
            Stmt::StmtFor(ref stmt) => stmt.pos,
            Stmt::StmtLoop(ref stmt) => stmt.pos,
            Stmt::StmtIf(ref stmt) => stmt.pos,
            Stmt::StmtExpr(ref stmt) => stmt.pos,
            Stmt::StmtBlock(ref stmt) => stmt.pos,
            Stmt::StmtBreak(ref stmt) => stmt.pos,
            Stmt::StmtContinue(ref stmt) => stmt.pos,
            Stmt::StmtReturn(ref stmt) => stmt.pos,
            Stmt::StmtThrow(ref stmt) => stmt.pos,
            Stmt::StmtDefer(ref stmt) => stmt.pos,
            Stmt::StmtDo(ref stmt) => stmt.pos,
            Stmt::StmtSpawn(ref stmt) => stmt.pos,
        }
    }

    pub fn span(&self) -> Span {
        match *self {
            Stmt::StmtVar(ref stmt) => stmt.span,
            Stmt::StmtWhile(ref stmt) => stmt.span,
            Stmt::StmtFor(ref stmt) => stmt.span,
            Stmt::StmtLoop(ref stmt) => stmt.span,
            Stmt::StmtIf(ref stmt) => stmt.span,
            Stmt::StmtExpr(ref stmt) => stmt.span,
            Stmt::StmtBlock(ref stmt) => stmt.span,
            Stmt::StmtBreak(ref stmt) => stmt.span,
            Stmt::StmtContinue(ref stmt) => stmt.span,
            Stmt::StmtReturn(ref stmt) => stmt.span,
            Stmt::StmtThrow(ref stmt) => stmt.span,
            Stmt::StmtDefer(ref stmt) => stmt.span,
            Stmt::StmtDo(ref stmt) => stmt.span,
            Stmt::StmtSpawn(ref stmt) => stmt.span,
        }
    }

    pub fn to_throw(&self) -> Option<&StmtThrowType> {
        match *self {
            Stmt::StmtThrow(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_throw(&self) -> bool {
        match *self {
            Stmt::StmtThrow(_) => true,
            _ => false,
        }
    }

    pub fn to_defer(&self) -> Option<&StmtDeferType> {
        match *self {
            Stmt::StmtDefer(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_defer(&self) -> bool {
        match *self {
            Stmt::StmtDefer(_) => true,
            _ => false,
        }
    }

    pub fn to_do(&self) -> Option<&StmtDoType> {
        match *self {
            Stmt::StmtDo(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_try(&self) -> bool {
        match *self {
            Stmt::StmtDo(_) => true,
            _ => false,
        }
    }

    pub fn to_var(&self) -> Option<&StmtVarType> {
        match *self {
            Stmt::StmtVar(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_var(&self) -> bool {
        match *self {
            Stmt::StmtVar(_) => true,
            _ => false,
        }
    }

    pub fn to_while(&self) -> Option<&StmtWhileType> {
        match *self {
            Stmt::StmtWhile(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_while(&self) -> bool {
        match *self {
            Stmt::StmtWhile(_) => true,
            _ => false,
        }
    }

    pub fn to_for(&self) -> Option<&StmtForType> {
        match *self {
            Stmt::StmtFor(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_for(&self) -> bool {
        match *self {
            Stmt::StmtFor(_) => true,
            _ => false,
        }
    }

    pub fn to_loop(&self) -> Option<&StmtLoopType> {
        match *self {
            Stmt::StmtLoop(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_loop(&self) -> bool {
        match *self {
            Stmt::StmtLoop(_) => true,
            _ => false,
        }
    }

    pub fn to_if(&self) -> Option<&StmtIfType> {
        match *self {
            Stmt::StmtIf(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_if(&self) -> bool {
        match *self {
            Stmt::StmtIf(_) => true,
            _ => false,
        }
    }

    pub fn to_expr(&self) -> Option<&StmtExprType> {
        match *self {
            Stmt::StmtExpr(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_expr(&self) -> bool {
        match *self {
            Stmt::StmtExpr(_) => true,
            _ => false,
        }
    }

    pub fn to_block(&self) -> Option<&StmtBlockType> {
        match *self {
            Stmt::StmtBlock(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_block(&self) -> bool {
        match *self {
            Stmt::StmtBlock(_) => true,
            _ => false,
        }
    }

    pub fn to_return(&self) -> Option<&StmtReturnType> {
        match *self {
            Stmt::StmtReturn(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_return(&self) -> bool {
        match *self {
            Stmt::StmtReturn(_) => true,
            _ => false,
        }
    }

    pub fn to_break(&self) -> Option<&StmtBreakType> {
        match *self {
            Stmt::StmtBreak(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_break(&self) -> bool {
        match *self {
            Stmt::StmtBreak(_) => true,
            _ => false,
        }
    }

    pub fn to_continue(&self) -> Option<&StmtContinueType> {
        match *self {
            Stmt::StmtContinue(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_continue(&self) -> bool {
        match *self {
            Stmt::StmtContinue(_) => true,
            _ => false,
        }
    }

    pub fn to_spawn(&self) -> Option<&StmtSpawnType> {
        match *self {
            Stmt::StmtSpawn(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_spawn(&self) -> bool {
        match *self {
            Stmt::StmtSpawn(_) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct StmtVarType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub name: Name,
    pub reassignable: bool,

    pub data_type: Option<Type>,
    pub expr: Option<Box<Expr>>,
}

#[derive(Clone, Debug)]
pub struct StmtForType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub name: Name,
    pub expr: Box<Expr>,
    pub block: Box<Stmt>,
}

#[derive(Clone, Debug)]
pub struct StmtWhileType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub cond: Box<Expr>,
    pub block: Box<Stmt>,
}

#[derive(Clone, Debug)]
pub struct StmtLoopType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub block: Box<Stmt>,
}

#[derive(Clone, Debug)]
pub struct StmtIfType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub cond: Box<Expr>,
    pub then_block: Box<Stmt>,
    pub else_block: Option<Box<Stmt>>,
}

#[derive(Clone, Debug)]
pub struct StmtExprType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub expr: Box<Expr>,
}

#[derive(Clone, Debug)]
pub struct StmtBlockType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub stmts: Vec<Box<Stmt>>,
}

#[derive(Clone, Debug)]
pub struct StmtReturnType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub expr: Option<Box<Expr>>,
}

#[derive(Clone, Debug)]
pub struct StmtBreakType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct StmtContinueType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct StmtThrowType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub expr: Box<Expr>,
}

#[derive(Clone, Debug)]
pub struct StmtDeferType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub expr: Box<Expr>,
}

#[derive(Clone, Debug)]
pub struct StmtDoType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub do_block: Box<Stmt>,
    pub catch_blocks: Vec<CatchBlock>,
    pub finally_block: Option<FinallyBlock>,
}

#[derive(Clone, Debug)]
pub struct StmtSpawnType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub expr: Box<Expr>,
}

#[derive(Clone, Debug)]
pub struct CatchBlock {
    pub id: NodeId,
    pub name: Name,
    pub pos: Position,
    pub span: Span,

    pub data_type: Type,
    pub block: Box<Stmt>,
}

impl CatchBlock {
    pub fn new(
        id: NodeId,
        name: Name,
        pos: Position,
        data_type: Type,
        block: Box<Stmt>,
    ) -> CatchBlock {
        CatchBlock {
            id: id,
            name: name,
            pos: pos,
            span: Span::invalid(),

            data_type: data_type,
            block: block,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FinallyBlock {
    pub block: Box<Stmt>,
}

impl FinallyBlock {
    pub fn new(block: Box<Stmt>) -> FinallyBlock {
        FinallyBlock { block: block }
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum UnOp {
    Plus,
    Neg,
    Not,
}

impl UnOp {
    pub fn as_str(&self) -> &'static str {
        match *self {
            UnOp::Plus => "+",
            UnOp::Neg => "-",
            UnOp::Not => "!",
        }
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum CmpOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Is,
    IsNot,
}

impl CmpOp {
    pub fn as_str(&self) -> &'static str {
        match *self {
            CmpOp::Eq => "==",
            CmpOp::Ne => "!=",
            CmpOp::Lt => "<",
            CmpOp::Le => "<=",
            CmpOp::Gt => ">",
            CmpOp::Ge => ">=",
            CmpOp::Is => "===",
            CmpOp::IsNot => "!==",
        }
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Cmp(CmpOp),
    Or,
    And,
    BitOr,
    BitAnd,
    BitXor,
    ShiftL,
    ArithShiftR,
    LogicalShiftR,
}

impl BinOp {
    pub fn as_str(&self) -> &'static str {
        match *self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Mod => "%",
            BinOp::Cmp(op) => op.as_str(),
            BinOp::Or => "||",
            BinOp::And => "&&",
            BinOp::BitOr => "|",
            BinOp::BitAnd => "&",
            BinOp::BitXor => "^",
            BinOp::ShiftL => "<<",
            BinOp::ArithShiftR => ">>",
            BinOp::LogicalShiftR => ">>>",
        }
    }

    pub fn is_compare(&self) -> bool {
        match *self {
            BinOp::Cmp(cmp) if cmp != CmpOp::Is && cmp != CmpOp::IsNot => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Expr {
    ExprUn(ExprUnType),
    ExprBin(ExprBinType),
    ExprLitChar(ExprLitCharType),
    ExprLitInt(ExprLitIntType),
    ExprLitFloat(ExprLitFloatType),
    ExprLitStr(ExprLitStrType),
    ExprLitBool(ExprLitBoolType),
    ExprIdent(ExprIdentType),
    ExprCall(ExprCallType),
    ExprTypeParam(ExprTypeParamType),
    ExprPath(ExprPathType),
    ExprDelegation(ExprDelegationType),
    ExprAssign(ExprAssignType),
    ExprDot(ExprDotType),
    ExprSelf(ExprSelfType),
    ExprSuper(ExprSuperType),
    ExprNil(ExprNilType),
    ExprConv(ExprConvType),
    ExprTry(ExprTryType),
    ExprLambda(ExprLambdaType),
}

impl Expr {
    pub fn create_un(id: NodeId, pos: Position, span: Span, op: UnOp, opnd: Box<Expr>) -> Expr {
        Expr::ExprUn(ExprUnType {
            id: id,
            pos: pos,
            span: span,

            op: op,
            opnd: opnd,
        })
    }

    pub fn create_try(
        id: NodeId,
        pos: Position,
        span: Span,
        expr: Box<Expr>,
        mode: TryMode,
    ) -> Expr {
        Expr::ExprTry(ExprTryType {
            id: id,
            pos: pos,
            span: span,

            expr: expr,
            mode: mode,
        })
    }

    pub fn create_bin(
        id: NodeId,
        pos: Position,
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    ) -> Expr {
        Expr::ExprBin(ExprBinType {
            id: id,
            pos: pos,
            span: Span::invalid(),

            op: op,
            lhs: lhs,
            rhs: rhs,
        })
    }

    pub fn create_conv(
        id: NodeId,
        pos: Position,
        object: Box<Expr>,
        data_type: Box<Type>,
        is: bool,
    ) -> Expr {
        Expr::ExprConv(ExprConvType {
            id: id,
            pos: pos,
            span: Span::invalid(),

            object: object,
            data_type: data_type,
            is: is,
        })
    }

    pub fn create_lit_char(id: NodeId, pos: Position, span: Span, value: char) -> Expr {
        Expr::ExprLitChar(ExprLitCharType {
            id: id,
            pos: pos,
            span: span,

            value: value,
        })
    }

    pub fn create_lit_int(
        id: NodeId,
        pos: Position,
        span: Span,
        value: u64,
        base: IntBase,
        suffix: IntSuffix,
    ) -> Expr {
        Expr::ExprLitInt(ExprLitIntType {
            id: id,
            pos: pos,
            span: span,

            value: value,
            base: base,
            suffix: suffix,
        })
    }

    pub fn create_lit_float(
        id: NodeId,
        pos: Position,
        span: Span,
        value: f64,
        suffix: FloatSuffix,
    ) -> Expr {
        Expr::ExprLitFloat(ExprLitFloatType {
            id: id,
            pos: pos,
            span: span,
            value: value,
            suffix: suffix,
        })
    }

    pub fn create_lit_str(id: NodeId, pos: Position, span: Span, value: String) -> Expr {
        Expr::ExprLitStr(ExprLitStrType {
            id: id,
            pos: pos,
            span: span,

            value: value,
        })
    }

    pub fn create_lit_bool(id: NodeId, pos: Position, span: Span, value: bool) -> Expr {
        Expr::ExprLitBool(ExprLitBoolType {
            id: id,
            pos: pos,
            span: span,

            value: value,
        })
    }

    pub fn create_this(id: NodeId, pos: Position, span: Span) -> Expr {
        Expr::ExprSelf(ExprSelfType {
            id: id,
            pos: pos,
            span: span,
        })
    }

    pub fn create_super(id: NodeId, pos: Position, span: Span) -> Expr {
        Expr::ExprSuper(ExprSuperType {
            id: id,
            pos: pos,
            span: span,
        })
    }

    pub fn create_nil(id: NodeId, pos: Position, span: Span) -> Expr {
        Expr::ExprNil(ExprNilType {
            id: id,
            pos: pos,
            span: span,
        })
    }

    pub fn create_ident(
        id: NodeId,
        pos: Position,
        span: Span,
        name: Name,
        type_params: Option<Vec<Type>>,
    ) -> Expr {
        Expr::ExprIdent(ExprIdentType {
            id: id,
            pos: pos,
            span: span,

            name: name,
            type_params: type_params,
        })
    }

    pub fn create_call(
        id: NodeId,
        pos: Position,
        span: Span,
        callee: Box<Expr>,
        args: Vec<Box<Expr>>,
    ) -> Expr {
        Expr::ExprCall(ExprCallType {
            id: id,
            pos: pos,
            span: span,

            callee: callee,
            args: args,
        })
    }

    pub fn create_type_param(
        id: NodeId,
        pos: Position,
        span: Span,
        callee: Box<Expr>,
        args: Vec<Type>,
    ) -> Expr {
        Expr::ExprTypeParam(ExprTypeParamType {
            id: id,
            pos: pos,
            span: span,

            callee: callee,
            args: args,
        })
    }

    pub fn create_path(
        id: NodeId,
        pos: Position,
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    ) -> Expr {
        Expr::ExprPath(ExprPathType {
            id: id,
            pos: pos,
            span: span,

            lhs: lhs,
            rhs: rhs,
        })
    }

    pub fn create_delegation(
        id: NodeId,
        pos: Position,
        ty: DelegationType,
        args: Vec<Box<Expr>>,
    ) -> Expr {
        Expr::ExprDelegation(ExprDelegationType {
            id: id,
            pos: pos,
            span: Span::invalid(),

            ty: ty,
            args: args,
        })
    }

    pub fn create_assign(id: NodeId, pos: Position, lhs: Box<Expr>, rhs: Box<Expr>) -> Expr {
        Expr::ExprAssign(ExprAssignType {
            id: id,
            pos: pos,
            span: Span::invalid(),

            lhs: lhs,
            rhs: rhs,
        })
    }

    pub fn create_dot(
        id: NodeId,
        pos: Position,
        span: Span,
        object: Box<Expr>,
        name: Name,
    ) -> Expr {
        Expr::ExprDot(ExprDotType {
            id: id,
            pos: pos,
            span: span,

            object: object,
            name: name,
        })
    }

    pub fn create_lambda(
        id: NodeId,
        pos: Position,
        span: Span,
        params: Vec<Param>,
        ret: Option<Box<Type>>,
        block: Box<Stmt>,
    ) -> Expr {
        Expr::ExprLambda(ExprLambdaType {
            id: id,
            pos: pos,
            span: span,

            params: params,
            ret: ret,
            block: block,
        })
    }

    pub fn to_un(&self) -> Option<&ExprUnType> {
        match *self {
            Expr::ExprUn(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_un(&self) -> bool {
        match *self {
            Expr::ExprUn(_) => true,
            _ => false,
        }
    }

    pub fn to_bin(&self) -> Option<&ExprBinType> {
        match *self {
            Expr::ExprBin(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_bin(&self) -> bool {
        match *self {
            Expr::ExprBin(_) => true,
            _ => false,
        }
    }

    pub fn to_assign(&self) -> Option<&ExprAssignType> {
        match *self {
            Expr::ExprAssign(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_assign(&self) -> bool {
        match *self {
            Expr::ExprAssign(_) => true,
            _ => false,
        }
    }

    pub fn to_ident(&self) -> Option<&ExprIdentType> {
        match *self {
            Expr::ExprIdent(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_ident(&self) -> bool {
        match *self {
            Expr::ExprIdent(_) => true,
            _ => false,
        }
    }

    pub fn to_call(&self) -> Option<&ExprCallType> {
        match *self {
            Expr::ExprCall(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_call(&self) -> bool {
        match *self {
            Expr::ExprCall(_) => true,
            _ => false,
        }
    }

    pub fn to_path(&self) -> Option<&ExprPathType> {
        match *self {
            Expr::ExprPath(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_path(&self) -> bool {
        match *self {
            Expr::ExprPath(_) => true,
            _ => false,
        }
    }

    pub fn to_type_param(&self) -> Option<&ExprTypeParamType> {
        match *self {
            Expr::ExprTypeParam(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_type_param(&self) -> bool {
        match *self {
            Expr::ExprTypeParam(_) => true,
            _ => false,
        }
    }

    pub fn to_lit_char(&self) -> Option<&ExprLitCharType> {
        match *self {
            Expr::ExprLitChar(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_lit_char(&self) -> bool {
        match *self {
            Expr::ExprLitChar(_) => true,
            _ => false,
        }
    }

    pub fn to_lit_int(&self) -> Option<&ExprLitIntType> {
        match *self {
            Expr::ExprLitInt(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_lit_int(&self) -> bool {
        match *self {
            Expr::ExprLitInt(_) => true,
            _ => false,
        }
    }

    pub fn to_lit_float(&self) -> Option<&ExprLitFloatType> {
        match *self {
            Expr::ExprLitFloat(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_lit_float(&self) -> bool {
        match *self {
            Expr::ExprLitFloat(_) => true,
            _ => false,
        }
    }

    pub fn to_lit_str(&self) -> Option<&ExprLitStrType> {
        match *self {
            Expr::ExprLitStr(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn to_lit_bool(&self) -> Option<&ExprLitBoolType> {
        match *self {
            Expr::ExprLitBool(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_lit_bool(&self) -> bool {
        match *self {
            Expr::ExprLitBool(_) => true,
            _ => false,
        }
    }

    pub fn is_lit_true(&self) -> bool {
        match *self {
            Expr::ExprLitBool(ref lit) if lit.value => true,
            _ => false,
        }
    }

    pub fn to_dot(&self) -> Option<&ExprDotType> {
        match *self {
            Expr::ExprDot(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_dot(&self) -> bool {
        match *self {
            Expr::ExprDot(_) => true,
            _ => false,
        }
    }

    pub fn to_delegation(&self) -> Option<&ExprDelegationType> {
        match *self {
            Expr::ExprDelegation(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_delegation(&self) -> bool {
        match *self {
            Expr::ExprDelegation(_) => true,
            _ => false,
        }
    }

    pub fn is_this(&self) -> bool {
        match *self {
            Expr::ExprSelf(_) => true,
            _ => false,
        }
    }

    pub fn is_super(&self) -> bool {
        match *self {
            Expr::ExprSuper(_) => true,
            _ => false,
        }
    }

    pub fn to_super(&self) -> Option<&ExprSuperType> {
        match *self {
            Expr::ExprSuper(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_nil(&self) -> bool {
        match *self {
            Expr::ExprNil(_) => true,
            _ => false,
        }
    }

    pub fn to_conv(&self) -> Option<&ExprConvType> {
        match *self {
            Expr::ExprConv(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_conv(&self) -> bool {
        match *self {
            Expr::ExprConv(_) => true,
            _ => false,
        }
    }

    pub fn to_try(&self) -> Option<&ExprTryType> {
        match *self {
            Expr::ExprTry(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_try(&self) -> bool {
        match *self {
            Expr::ExprTry(_) => true,
            _ => false,
        }
    }

    pub fn to_lambda(&self) -> Option<&ExprLambdaType> {
        match *self {
            Expr::ExprLambda(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_lambda(&self) -> bool {
        match self {
            &Expr::ExprLambda(_) => true,
            _ => false,
        }
    }

    pub fn pos(&self) -> Position {
        match *self {
            Expr::ExprUn(ref val) => val.pos,
            Expr::ExprBin(ref val) => val.pos,
            Expr::ExprLitChar(ref val) => val.pos,
            Expr::ExprLitInt(ref val) => val.pos,
            Expr::ExprLitFloat(ref val) => val.pos,
            Expr::ExprLitStr(ref val) => val.pos,
            Expr::ExprLitBool(ref val) => val.pos,
            Expr::ExprIdent(ref val) => val.pos,
            Expr::ExprAssign(ref val) => val.pos,
            Expr::ExprCall(ref val) => val.pos,
            Expr::ExprTypeParam(ref val) => val.pos,
            Expr::ExprPath(ref val) => val.pos,
            Expr::ExprDelegation(ref val) => val.pos,
            Expr::ExprDot(ref val) => val.pos,
            Expr::ExprSelf(ref val) => val.pos,
            Expr::ExprSuper(ref val) => val.pos,
            Expr::ExprNil(ref val) => val.pos,
            Expr::ExprConv(ref val) => val.pos,
            Expr::ExprTry(ref val) => val.pos,
            Expr::ExprLambda(ref val) => val.pos,
        }
    }

    pub fn span(&self) -> Span {
        match *self {
            Expr::ExprUn(ref val) => val.span,
            Expr::ExprBin(ref val) => val.span,
            Expr::ExprLitChar(ref val) => val.span,
            Expr::ExprLitInt(ref val) => val.span,
            Expr::ExprLitFloat(ref val) => val.span,
            Expr::ExprLitStr(ref val) => val.span,
            Expr::ExprLitBool(ref val) => val.span,
            Expr::ExprIdent(ref val) => val.span,
            Expr::ExprAssign(ref val) => val.span,
            Expr::ExprCall(ref val) => val.span,
            Expr::ExprTypeParam(ref val) => val.span,
            Expr::ExprPath(ref val) => val.span,
            Expr::ExprDelegation(ref val) => val.span,
            Expr::ExprDot(ref val) => val.span,
            Expr::ExprSelf(ref val) => val.span,
            Expr::ExprSuper(ref val) => val.span,
            Expr::ExprNil(ref val) => val.span,
            Expr::ExprConv(ref val) => val.span,
            Expr::ExprTry(ref val) => val.span,
            Expr::ExprLambda(ref val) => val.span,
        }
    }

    pub fn id(&self) -> NodeId {
        match *self {
            Expr::ExprUn(ref val) => val.id,
            Expr::ExprBin(ref val) => val.id,
            Expr::ExprLitChar(ref val) => val.id,
            Expr::ExprLitInt(ref val) => val.id,
            Expr::ExprLitFloat(ref val) => val.id,
            Expr::ExprLitStr(ref val) => val.id,
            Expr::ExprLitBool(ref val) => val.id,
            Expr::ExprIdent(ref val) => val.id,
            Expr::ExprAssign(ref val) => val.id,
            Expr::ExprCall(ref val) => val.id,
            Expr::ExprTypeParam(ref val) => val.id,
            Expr::ExprPath(ref val) => val.id,
            Expr::ExprDelegation(ref val) => val.id,
            Expr::ExprDot(ref val) => val.id,
            Expr::ExprSelf(ref val) => val.id,
            Expr::ExprSuper(ref val) => val.id,
            Expr::ExprNil(ref val) => val.id,
            Expr::ExprConv(ref val) => val.id,
            Expr::ExprTry(ref val) => val.id,
            Expr::ExprLambda(ref val) => val.id,
        }
    }
}

#[derive(Clone, Debug)]
pub struct StructArg {
    pub id: NodeId,
    pub pos: Position,
    pub name: Name,
    pub expr: Box<Expr>,
}

#[derive(Clone, Debug)]
pub struct ExprConvType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub object: Box<Expr>,
    pub is: bool,
    pub data_type: Box<Type>,
}

#[derive(Clone, Debug)]
pub struct ExprTryType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub expr: Box<Expr>,
    pub mode: TryMode,
}

#[derive(Clone, Debug)]
pub enum TryMode {
    Normal,
    Else(Box<Expr>),
    Opt,
    Force,
}

impl TryMode {
    pub fn is_normal(&self) -> bool {
        match self {
            &TryMode::Normal => true,
            _ => false,
        }
    }

    pub fn is_else(&self) -> bool {
        match self {
            &TryMode::Else(_) => true,
            _ => false,
        }
    }

    pub fn is_force(&self) -> bool {
        match self {
            &TryMode::Force => true,
            _ => false,
        }
    }

    pub fn is_opt(&self) -> bool {
        match self {
            &TryMode::Opt => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ExprDelegationType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub ty: DelegationType, // true for this class, false for super class
    pub args: Vec<Box<Expr>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DelegationType {
    This,
    Super,
}

impl DelegationType {
    pub fn is_this(&self) -> bool {
        match *self {
            DelegationType::This => true,
            _ => false,
        }
    }

    pub fn is_super(&self) -> bool {
        match *self {
            DelegationType::Super => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ExprUnType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub op: UnOp,
    pub opnd: Box<Expr>,
}

#[derive(Clone, Debug)]
pub struct ExprBinType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub op: BinOp,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

#[derive(Clone, Debug)]
pub struct ExprLitCharType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub value: char,
}

#[derive(Clone, Debug)]
pub struct ExprLitIntType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub value: u64,
    pub base: IntBase,
    pub suffix: IntSuffix,
}

#[derive(Clone, Debug)]
pub struct ExprLitFloatType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub value: f64,
    pub suffix: FloatSuffix,
}

#[derive(Clone, Debug)]
pub struct ExprLitStrType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub value: String,
}

#[derive(Clone, Debug)]
pub struct ExprLitBoolType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub value: bool,
}

#[derive(Clone, Debug)]
pub struct ExprSuperType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ExprSelfType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ExprNilType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ExprIdentType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub name: Name,
    pub type_params: Option<Vec<Type>>,
}

#[derive(Clone, Debug)]
pub struct ExprLambdaType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub params: Vec<Param>,
    pub ret: Option<Box<Type>>,
    pub block: Box<Stmt>,
}

#[derive(Clone, Debug)]
pub struct ExprCallType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub callee: Box<Expr>,
    pub args: Vec<Box<Expr>>,
}

impl ExprCallType {
    pub fn object(&self) -> Option<&Expr> {
        if let Some(type_param) = self.callee.to_type_param() {
            if let Some(dot) = type_param.callee.to_dot() {
                Some(&dot.object)
            } else {
                None
            }
        } else if let Some(dot) = self.callee.to_dot() {
            Some(&dot.object)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub struct ExprTypeParamType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub callee: Box<Expr>,
    pub args: Vec<Type>,
}

#[derive(Clone, Debug)]
pub struct ExprAssignType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

#[derive(Clone, Debug)]
pub struct ExprPathType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

#[derive(Clone, Debug)]
pub struct ExprDotType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub object: Box<Expr>,
    pub name: Name,
}
