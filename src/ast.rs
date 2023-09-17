use crate::lex::{val::ObjectVal, Token};

use thiserror::Error;

use std::rc::Rc;

use crate::lex::val;

#[derive(Debug, Clone)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping(Box<Expr>),
    Literal(val::ObjectVal),
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
}

impl Expr {
    pub fn walk<T, R>(&self, visitor: &mut T) -> anyhow::Result<R>
    where
        T: AstWalker<R>,
    {
        visitor.visit_expr(self)
    }
}

pub struct AstStringify;

pub trait AstWalker<T> {
    fn visit_expr(&mut self, expr: &Expr) -> anyhow::Result<T>;
}

#[derive(Error, Debug)]
pub enum AstWalkError {
    #[error("Runtime Error :: {token} => {message}")]
    RuntimeError { token: Token, message: String },
    #[error("Type Error :: {value} => {message}")]
    TypeError { value: ObjectVal, message: String },
}

impl AstStringify {
    pub fn stringify(&mut self, e: &Expr) -> anyhow::Result<String> {
        e.walk(self)
    }

    pub fn lispify(&mut self, name: &str, exprs: &[&Expr]) -> anyhow::Result<String> {
        let mut result = String::new();
        result.push_str(&format!("({name}"));
        for expr in exprs {
            result.push_str(&format!(" {}", expr.walk(self)?));
        }
        result.push_str(")");
        Ok(result)
    }
}

impl AstWalker<String> for AstStringify {
    fn visit_expr(&mut self, expr: &Expr) -> anyhow::Result<String> {
        match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => self.lispify(&operator.lexeme, &[left.as_ref(), right.as_ref()]),
            Expr::Grouping(exp) => self.lispify("group", &[&exp.as_ref()]),
            Expr::Literal(lit) => match lit {
                crate::lex::val::ObjectVal::Unit => Ok("nil".into()),

                _ => Ok(lit.to_string()),
            },
            Expr::Unary { operator, right } => self.lispify(&operator.lexeme, &[&right.as_ref()]),
        }
    }
}
