use crate::{
    ast::{self, AstWalkError, AstWalker},
    lex::{
        val::{self, ObjectVal},
        Token, TokenType,
    },
};
use anyhow::*;

pub struct Interpreter;

impl Interpreter {
    pub fn eval(expr: &ast::Expr) -> anyhow::Result<ObjectVal> {
        expr.walk(&mut Interpreter)
    }
}

impl AstWalker<val::ObjectVal> for Interpreter {
    fn visit_expr(&mut self, expr: &ast::Expr) -> anyhow::Result<val::ObjectVal> {
        match expr {
            ast::Expr::Binary {
                left,
                operator,
                right,
            } => {
                let lvalue = left.walk(self)?;
                let rvalue = right.walk(self)?;
                match operator.ty {
                    TokenType::Minus => eval_sub(&lvalue, operator, &rvalue),
                    TokenType::Plus => eval_plus(&lvalue, operator, &rvalue),
                    TokenType::ForwardSlash => eval_div(&lvalue, operator, &rvalue),
                    TokenType::Star => eval_mul(&lvalue, operator, &rvalue),
                    TokenType::Lt => eval_lt(&lvalue, operator, &rvalue),
                    TokenType::Le => eval_le(&lvalue, operator, &rvalue),
                    TokenType::Gt => eval_gt(&lvalue, operator, &rvalue),
                    TokenType::Ge => eval_ge(&lvalue, operator, &rvalue),
                    TokenType::EqualEqual => Ok(ObjectVal::Boolean(lvalue == rvalue)),
                    TokenType::BangEqual => Ok(ObjectVal::Boolean(lvalue != rvalue)),
                    _ => bail!(
                        "{}",
                        AstWalkError::RuntimeError {
                            token: operator.clone(),
                            message: "Unknown binary operator found".into()
                        }
                    ),
                }
            }
            ast::Expr::Grouping(e) => Ok(e.walk(self)?),
            ast::Expr::Literal(lit) => Ok(lit.clone()),
            ast::Expr::Unary { operator, right } => {
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

// TODO :: Refactor these eval_* functions into a single macro that can print out this code, or at
// least define the eval_* functions with highly similar function bodies
pub fn eval_minus(minus_op: &Token, value: &ObjectVal) -> anyhow::Result<val::ObjectVal> {
    let num = value.as_number().map_err(|e| AstWalkError::RuntimeError {
        token: minus_op.clone(),
        message: format!("Operator must be a number, {}", e),
    })?;
    Ok(ObjectVal::Number(-num))
}

pub fn eval_le(
    left: &ObjectVal,
    operator: &Token,
    right: &ObjectVal,
) -> anyhow::Result<val::ObjectVal> {
    match left {
        ObjectVal::Number(ln) => {
            let rn = right.as_number().map_err(|e| AstWalkError::RuntimeError {
                token: operator.clone(),
                message: format!(
                    "mismatched less-than-equal operator: '{} < {}', {}",
                    left.type_string(),
                    right.type_string(),
                    e
                ),
            })?;
            Ok(ObjectVal::Boolean(ln < &rn))
        }
        _ => bail!(
            "{}",
            AstWalkError::RuntimeError {
                token: operator.clone(),
                message: format!(
                    "Lefthand side of multiplication operator must be a number, got: {}",
                    left.type_string(),
                )
            }
        ),
    }
}

pub fn eval_lt(
    left: &ObjectVal,
    operator: &Token,
    right: &ObjectVal,
) -> anyhow::Result<val::ObjectVal> {
    match left {
        ObjectVal::Number(ln) => {
            let rn = right.as_number().map_err(|e| AstWalkError::RuntimeError {
                token: operator.clone(),
                message: format!(
                    "mismatched less-than operator: '{} < {}', {}",
                    left.type_string(),
                    right.type_string(),
                    e
                ),
            })?;
            Ok(ObjectVal::Boolean(ln > &rn))
        }
        _ => bail!(
            "{}",
            AstWalkError::RuntimeError {
                token: operator.clone(),
                message: format!(
                    "Lefthand side of multiplication operator must be a number, got: {}",
                    left.type_string(),
                )
            }
        ),
    }
}

pub fn eval_ge(
    left: &ObjectVal,
    operator: &Token,
    right: &ObjectVal,
) -> anyhow::Result<val::ObjectVal> {
    match left {
        ObjectVal::Number(ln) => {
            let rn = right.as_number().map_err(|e| AstWalkError::RuntimeError {
                token: operator.clone(),
                message: format!(
                    "mismatched greater-than-equal operator: '{} >= {}', {}",
                    left.type_string(),
                    right.type_string(),
                    e
                ),
            })?;
            Ok(ObjectVal::Boolean(ln >= &rn))
        }
        _ => bail!(
            "{}",
            AstWalkError::RuntimeError {
                token: operator.clone(),
                message: format!(
                    "Lefthand side of multiplication operator must be a number, got: {}",
                    left.type_string(),
                )
            }
        ),
    }
}

pub fn eval_gt(
    left: &ObjectVal,
    operator: &Token,
    right: &ObjectVal,
) -> anyhow::Result<val::ObjectVal> {
    match left {
        ObjectVal::Number(ln) => {
            let rn = right.as_number().map_err(|e| AstWalkError::RuntimeError {
                token: operator.clone(),
                message: format!(
                    "mismatched greater-than operator: '{} > {}', {}",
                    left.type_string(),
                    right.type_string(),
                    e
                ),
            })?;
            Ok(ObjectVal::Boolean(ln > &rn))
        }
        _ => bail!(
            "{}",
            AstWalkError::RuntimeError {
                token: operator.clone(),
                message: format!(
                    "Lefthand side of multiplication operator must be a number, got: {}",
                    left.type_string(),
                )
            }
        ),
    }
}

pub fn eval_mul(
    left: &ObjectVal,
    operator: &Token,
    right: &ObjectVal,
) -> anyhow::Result<val::ObjectVal> {
    match left {
        ObjectVal::Number(ln) => {
            let rn = right.as_number().map_err(|e| AstWalkError::RuntimeError {
                token: operator.clone(),
                message: format!(
                    "mismatched multiplication operator: '{} * {}', {}",
                    left.type_string(),
                    right.type_string(),
                    e
                ),
            })?;
            Ok(ObjectVal::Number(ln * rn))
        }
        _ => bail!(
            "{}",
            AstWalkError::RuntimeError {
                token: operator.clone(),
                message: format!(
                    "Lefthand side of multiplication operator must be a number, got: {}",
                    left.type_string(),
                )
            }
        ),
    }
}

pub fn eval_div(
    left: &ObjectVal,
    operator: &Token,
    right: &ObjectVal,
) -> anyhow::Result<val::ObjectVal> {
    match left {
        ObjectVal::Number(ln) => {
            let rn = right.as_number().map_err(|e| AstWalkError::RuntimeError {
                token: operator.clone(),
                message: format!(
                    "mismatched division operator: '{} / {}', {}",
                    left.type_string(),
                    right.type_string(),
                    e
                ),
            })?;
            Ok(ObjectVal::Number(ln / rn))
        }
        _ => bail!(
            "{}",
            AstWalkError::RuntimeError {
                token: operator.clone(),
                message: format!(
                    "Lefthand side of division operator must be a number, got: {}",
                    left.type_string(),
                )
            }
        ),
    }
}

pub fn eval_sub(
    left: &ObjectVal,
    operator: &Token,
    right: &ObjectVal,
) -> anyhow::Result<val::ObjectVal> {
    match left {
        ObjectVal::Number(ln) => {
            let rn = right.as_number().map_err(|e| AstWalkError::RuntimeError {
                token: operator.clone(),
                message: format!(
                    "mismatched subtraction operator: '{} - {}', {}",
                    left.type_string(),
                    right.type_string(),
                    e
                ),
            })?;
            Ok(ObjectVal::Number(ln - rn))
        }
        _ => bail!(
            "{}",
            AstWalkError::RuntimeError {
                token: operator.clone(),
                message: format!(
                    "Lefthand side of subtraction operator must be a number, got: {}",
                    left.type_string(),
                )
            }
        ),
    }
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
                    "mismatched addition operator: '{} + {}', {}",
                    left.type_string(),
                    right.type_string(),
                    e
                ),
            })?;
            Ok(ObjectVal::Number(ln + rn))
        }
        ObjectVal::String(ls) => {
            let rs = right.as_string().map_err(|e| AstWalkError::RuntimeError {
                token: operator.clone(),
                message: format!(
                    "mismatched addition operator: '{} + {}', {}",
                    left.type_string(),
                    right.type_string(),
                    e
                ),
            })?;
            Ok(ObjectVal::String(ls.clone() + &rs))
        }
        _ => todo!(),
    }
}
