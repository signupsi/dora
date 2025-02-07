use crate::class::TypeParams;
use crate::ty::BuiltinType;
use crate::typeck::expr::{check_lit_float, check_lit_int, lookup_method};
use crate::vm::{ConstData, ConstValue, VM};

use dora_parser::ast::Expr::*;
use dora_parser::ast::*;
use dora_parser::error::msg::Msg;

pub struct ConstCheck<'a, 'ast: 'a> {
    pub vm: &'a VM<'ast>,
    pub xconst: &'a ConstData,
    pub negative_expr_id: NodeId,
}

impl<'a, 'ast> ConstCheck<'a, 'ast> {
    pub fn check_expr(&mut self, expr: &Expr) -> (BuiltinType, ConstValue) {
        let (ty, lit) = match expr {
            &ExprLitChar(ref expr) => (BuiltinType::Char, ConstValue::Char(expr.value)),
            &ExprLitInt(ref expr) => {
                let (ty, val) = check_lit_int(self.vm, expr, self.negative_expr_id);
                (ty, ConstValue::Int(val))
            }
            &ExprLitFloat(ref expr) => {
                let (ty, val) = check_lit_float(self.vm, expr, self.negative_expr_id);
                (ty, ConstValue::Float(val))
            }
            &ExprLitBool(ref expr) => (BuiltinType::Bool, ConstValue::Bool(expr.value)),

            &ExprUn(ref expr) if expr.op == UnOp::Neg => {
                if self.negative_expr_id != expr.id {
                    self.negative_expr_id = expr.opnd.id();
                }

                let (ty, val) = self.check_expr(&expr.opnd);
                let name = self.vm.interner.intern("unaryMinus");

                if lookup_method(
                    self.vm,
                    ty,
                    false,
                    name,
                    &[],
                    &TypeParams::empty(),
                    Some(ty),
                )
                .is_none()
                {
                    let ty = ty.name(self.vm);
                    let msg = Msg::UnOpType(expr.op.as_str().into(), ty);

                    self.vm.diag.lock().report_without_path(expr.pos, msg);
                }

                return (ty, val);
            }

            _ => {
                let msg = Msg::ConstValueExpected;
                self.vm.diag.lock().report_without_path(expr.pos(), msg);
                return (BuiltinType::Error, ConstValue::None);
            }
        };

        if !self.xconst.ty.allows(self.vm, ty) {
            let name = self.vm.interner.str(self.xconst.name).to_string();
            let const_ty = self.xconst.ty.name(self.vm);
            let ty = ty.name(self.vm);
            let msg = Msg::AssignType(name, const_ty, ty);
            self.vm.diag.lock().report_without_path(expr.pos(), msg);
        }

        (ty, lit)
    }
}
