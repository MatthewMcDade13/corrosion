#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    Print,
    FatArrow,
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    ForwardSlash,
    Star,
    Bang,
    Equal,
    BangEqual,
    EqualEqual,
    Gt,
    Ge,
    Lt,
    Le,
    Ident,
    String,
    Number,
    And,
    Struct,
    Trait,
    Impl,
    Else,
    False,
    True,
    Fn,
    If,

    Unit,
    Or,
    Return,
    Super,
    ThisSelf,
    Let,
    Const,
    Eof,
    Loop,
    For,
    While,
    Break,
    Switch,
    Continue,
    Comment,
    Unknown,
    Colon,
    DoubleColon,
}

use anyhow::*;
use std::fmt;

use crate::ast::AstWalkError;

#[derive(Debug, Clone)]
pub struct Token {
    pub ty: TokenType,
    pub literal: Value,
    pub line: u32,
    pub lexeme: String,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            ty,
            literal,
            lexeme,
            line,
        } = self;
        write!(f, "LineNo:{line} {ty:?} :: {lexeme} :: {literal:?}")
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Unit,
    // use an Rc for object so we can keep ObjectVal as small as possible
    // OR we can have the language ast::Object wrap a Box/Rc, which is probably more
    // preferrable
    // Obj(Rc<ast::Object>),
}

impl Value {
    pub fn as_number(&self) -> anyhow::Result<f64> {
        let value = self.clone();
        if let Self::Number(n) = value {
            Ok(n)
        } else {
            let type_str = value.type_string();
            bail!(
                "{}",
                AstWalkError::TypeError {
                    value,
                    message: format!("Expected Number, got: {}", type_str)
                }
            )
        }
    }

    pub fn as_string(&self) -> anyhow::Result<String> {
        let value = self.clone();
        if let Self::String(string) = value {
            Ok(string)
        } else {
            let type_str = value.type_string();
            bail!(
                "{}",
                AstWalkError::TypeError {
                    value,
                    message: format!("Expected String, got: {}", type_str)
                }
            )
        }
    }

    pub fn as_bool(&self) -> anyhow::Result<bool> {
        let value = self.clone();
        if let Self::Boolean(b) = value {
            Ok(b)
        } else {
            let type_str = value.type_string();
            bail!(
                "{}",
                AstWalkError::TypeError {
                    value,
                    message: format!("Expected Boolean, got: {}", type_str)
                }
            )
        }
    }

    pub fn type_string(&self) -> String {
        match self {
            Value::Number(_) => "Number".into(),
            Value::String(_) => "String".into(),
            Value::Boolean(_) => "Boolean".into(),
            Value::Unit => "Unit".into(),
        }
    }

    pub const fn is_str(&self) -> bool {
        if let Self::String(_) = self {
            true
        } else {
            false
        }
    }
    pub const fn is_bool(&self) -> bool {
        if let Self::Boolean(_) = self {
            true
        } else {
            false
        }
    }

    pub const fn is_unit(&self) -> bool {
        if let Self::Unit = self {
            true
        } else {
            false
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Value::Number(left) => {
                if let Value::Number(right) = other {
                    left == right
                } else {
                    false
                }
            }
            Value::String(left) => {
                if let Value::String(right) = other {
                    left == right
                } else {
                    false
                }
            }
            Value::Boolean(left) => {
                if let Value::Boolean(right) = other {
                    left == right
                } else {
                    false
                }
            }
            Value::Unit => {
                if let Value::Unit = other {
                    true
                } else {
                    false
                }
            }
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Self::Unit
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.clone(),
            Value::Boolean(b) => b.to_string(),
            Value::Unit => String::from("()"),
        };
        write!(f, "{}", str)
    }
}
