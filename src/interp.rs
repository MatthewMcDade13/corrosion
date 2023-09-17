use crate::{
    ast::{self, AstWalkError, AstWalker},
    lex::{
        val::{self, ObjectVal},
        Token, TokenType,
    },
};
use anyhow::*;

pub struct Interpreter;

impl ast::AstWalker<val::ObjectVal> for Interpreter {
    fn visit_expr(&mut self, expr: &ast::expr::Expr) -> anyhow::Result<val::ObjectVal> {
        match expr {
            ast::expr::Expr::Binary {
                left,
                operator,
                right,
            } => todo!(),
            ast::expr::Expr::Grouping(e) => Ok(e.walk(self)?),
            ast::expr::Expr::Literal(lit) => Ok(lit.clone()),
            ast::expr::Expr::Unary { operator, right } => {
                let value = right.walk(self)?;
                match operator.ty {
                    TokenType::Minus => eval_minus(operator, &value),
                    _ => {
                        bail!(
                            "{}",
                            AstWalkError::RuntimeError {
                                token: operator.clone(),
                                message: "Unknown token".into()
                            }
                        )
                    }
                }
            }
        }
    }
}

pub fn eval_minus(minus_op: &Token, value: &ObjectVal) -> anyhow::Result<val::ObjectVal> {
    let num = value.as_number().map_err(|e| AstWalkError::RuntimeError {
        token: minus_op.clone(),
        message: format!("Operator must be a number, {}", e),
    })?;
    Ok(ObjectVal::Number(-num))
}

pub fn eval_plus(
    left: &ObjectVal,
    operator: &Token,
    right: &ObjectVal,
) -> anyhow::Result<val::ObjectVal> {
    match left {
        ObjectVal::Number(ln) => {
            let rn = right.as_number().map_err(|e| AstWalkError::RuntimeError {
                token: operator.clone(),
                message: format!(
                    "mismatched addition operator, '{} + {}'",
                    left.type_string(),
                    right.type_string()
                ),
            })?;
            Ok(ObjectVal::Number(ln + rn))
        }
        ObjectVal::String(ls) => {
            let rs = right.as_string().map_err(|e| AstWalkError::RuntimeError {
                token: operator.clone(),
                message: format!(
                    "mismatched addition operator, '{} + {}'",
                    left.type_string(),
                    right.type_string()
                ),
            })?;
            Ok(ObjectVal::String(ls.clone() + &rs))
        }
        _ => todo!(),
    }
}
