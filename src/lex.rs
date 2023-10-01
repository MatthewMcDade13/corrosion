use anyhow::*;
use std::{collections::HashMap, fs::File, path::Display};

#[derive(Debug, Clone)]
pub struct LexResult {
    pub tokens: Vec<Token>,
    pub errors: Vec<String>,
}

impl ToString for LexResult {
    fn to_string(&self) -> String {
        let mut string = String::new();
        let ts = self.tokens.iter().fold(String::new(), |mut acc, curr| {
            acc.push_str(" | ");
            acc.push_str(&curr.to_string());
            acc
        });
        let es = self.errors.iter().fold(String::new(), |mut acc, curr| {
            acc.push_str(" | ");
            acc.push_str(&curr);
            acc
        });

        string.push_str("TOKENS \n");
        string.push_str(&ts);
        string.push_str("\n");
        string.push_str("ERRORS \n");
        string.push_str(&es);
        string
    }
}

impl LexResult {
    pub fn empty() -> Self {
        Self {
            tokens: Vec::new(),
            errors: Vec::new(),
        }
    }
}

impl Default for LexResult {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Clone)]
pub struct Lexer {
    source: Vec<u8>,
    source_str: String,
    cursor: Cursor,
    tokens: Vec<Token>,
    errors: Vec<String>,
}
use phf::phf_map;

use crate::{
    sys::{is_alpha, is_alpha_numeric, is_digit},
    value::{Object, Token, TokenType, Value},
};

pub static KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    "struct" => TokenType::Struct,
    "trait" => TokenType::Trait,
    "impl" => TokenType::Impl,
    "if" => TokenType::If,
    "else" => TokenType::Else,
    "true" => TokenType::True,
    "false" => TokenType::False,
    "fn" => TokenType::Fn,
    "nil" => TokenType::Nil,
    "and" => TokenType::And,
    "or" => TokenType::Or,
    "return" => TokenType::Return,
    "super" => TokenType::Super,
    "self" => TokenType::ThisSelf,
    "let" => TokenType::Let,
    "const" => TokenType::Const,
    "loop" => TokenType::Loop,
    "for" => TokenType::For,
    "while" => TokenType::While,
    "break" => TokenType::Break,
    "switch" => TokenType::Switch,
    "continue" => TokenType::Continue,
    "print" => TokenType::Print,
};

impl Lexer {
    pub fn scan_tokens_file(filepath: &str) -> anyhow::Result<LexResult> {
        let source = std::fs::read_to_string(filepath)?;

        let s = Self::scan_tokens(source.trim());
        Ok(s)
    }
    pub fn scan_tokens(source_str: &str) -> LexResult {
        let mut lex = Self::new(source_str);
        while !lex.is_cursor_at_end() {
            let c = lex.next_token();
            let (ty, literal) = match c {
                '{' => (TokenType::LeftBrace, None),
                '}' => (TokenType::RightBrace, None),
                '(' => (TokenType::LeftParen, None),
                ')' => (TokenType::RightParen, None),

                ',' => (TokenType::Comma, None),
                '.' => (TokenType::Dot, None),
                '-' => (TokenType::Minus, None),
                '+' => (TokenType::Plus, None),

                ';' => (TokenType::Semicolon, None),
                ':' => (
                    if lex.match_next(':') {
                        TokenType::DoubleColon
                    } else {
                        TokenType::Colon
                    },
                    None,
                ),
                '*' => (TokenType::Star, None),
                '!' => (
                    if lex.match_next('=') {
                        TokenType::BangEqual
                    } else {
                        TokenType::Bang
                    },
                    None,
                ),

                '=' => (
                    if lex.match_next('=') {
                        TokenType::EqualEqual
                    } else if lex.match_next('>') {
                        TokenType::FatArrow
                    } else {
                        TokenType::Equal
                    },
                    None,
                ),

                '<' => (
                    if lex.match_next('=') {
                        TokenType::Le
                    } else {
                        TokenType::Lt
                    },
                    None,
                ),

                '>' => (
                    if lex.match_next('=') {
                        TokenType::Ge
                    } else {
                        TokenType::Gt
                    },
                    None,
                ),
                '/' => {
                    let ty = if lex.match_next('/') {
                        while !lex.is_cursor_at_end() && lex.peek() != '\n' {
                            lex.advance_cursor(1);
                        }
                        TokenType::Comment
                    } else {
                        TokenType::ForwardSlash
                    };
                    (ty, None)
                }
                '"' => (TokenType::String, lex.select_string()),
                _ => {
                    if is_digit(c) {
                        (TokenType::Number, lex.select_number())
                    } else if is_alpha(c) {
                        (lex.select_ident(), None)
                    } else {
                        lex.errors.push(format!(
                            "{} :: Unexpected Character - {}",
                            lex.cursor.lineno, c
                        ));
                        (TokenType::Unknown, None)
                    }
                }
            };
            let token = lex.cursor.to_token(source_str, ty, literal);
            lex.tokens.push(token);
        }
        lex.tokens.push(Token {
            ty: TokenType::Eof,
            literal: Value::Nil,
            line: 0,
            lexeme: "\0".into(),
        });
        LexResult {
            tokens: lex.tokens,
            errors: lex.errors,
        }
    }

    #[inline]
    fn advance_cursor(&mut self, n: usize) {
        self.cursor.i += n;
    }

    fn select_number(&mut self) -> Option<Value> {
        while is_digit(self.peek()) {
            self.advance_cursor(1);
        }

        if self.peek() == '.' && is_digit(self.peekn(1)) {
            self.advance_cursor(1);
            while is_digit(self.peek()) {
                self.advance_cursor(1);
            }
        }
        if let std::result::Result::Ok(value) =
            self.source_str[self.cursor.start..self.cursor.i].parse::<f64>()
        {
            Some(Value::Number(value))
        } else {
            self.errors
                .push(format!("{} :: Error parsing number", self.cursor.lineno));
            None
        }
    }
    fn select_ident(&mut self) -> TokenType {
        while is_alpha_numeric(self.peek()) {
            self.advance_cursor(1);
        }
        let value = self.source_str[self.cursor.start..self.cursor.i].to_string();

        if let Some(ty) = KEYWORDS.get(&value) {
            *ty
        } else {
            TokenType::Ident
        }
    }

    fn select_string(&mut self) -> Option<Value> {
        while !self.is_cursor_at_end() && self.peek() != '"' {
            if self.peek() == '\n' {
                self.cursor.lineno += 1
            }
            self.advance_cursor(1);
        }
        if self.is_cursor_at_end() {
            self.errors.push(format!(
                "{line} :: {message}",
                line = self.cursor.lineno,
                message = "Unterminated string"
            ));
            None
        } else {
            self.advance_cursor(1);
            let Cursor { start, i, .. } = self.cursor;
            // snip double quotes on ends of string
            let value = self.source_str[(start + 1)..(i - 1)].to_string();
            Some(Value::Obj(Object::String(value)))
        }
    }

    fn is_cursor_at_end(&self) -> bool {
        self.cursor.i >= self.source.len()
    }

    fn match_next(&mut self, c: char) -> bool {
        if self.is_cursor_at_end() || self.peek() != c {
            false
        } else {
            self.advance_cursor(1);
            true
        }
    }

    fn peek(&self) -> char {
        self.source[self.cursor.i] as char
    }
    fn peekn(&self, n: usize) -> char {
        let ci = self.cursor.i + n;
        assert!(
            ci < self.source.len(),
            "cursor index out of range of source string buffer"
        );
        self.source[ci] as char
    }
    fn next_token(&mut self) -> char {
        while !self.is_cursor_at_end() {
            let c = self.peek();
            if c.is_whitespace() {
                if c == '\n' {
                    self.cursor.lineno += 1;
                }
                self.advance_cursor(1);
            } else {
                self.cursor.start = self.cursor.i;
                self.advance_cursor(1);
                return c;
            }
        }
        '\0'
    }
    fn new(source_str: &str) -> Self {
        Self {
            source: source_str.as_bytes().to_vec(),
            source_str: String::from(source_str),
            tokens: Vec::new(),
            errors: Vec::new(),
            cursor: Cursor::default(),
        }
    }
    fn from_file(filename: &str) -> anyhow::Result<Self> {
        let source_str = std::fs::read_to_string(filename)?;
        let s = Self::new(source_str.as_ref());
        Ok(s)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Cursor {
    pub start: usize,
    pub i: usize,
    pub lineno: u32,
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            lineno: 1,
            start: 0,
            i: 0,
        }
    }

    pub fn to_token(&self, source: &str, ty: TokenType, literal: Option<Value>) -> Token {
        Token {
            ty,
            literal: literal.unwrap_or(Value::Nil),
            line: self.lineno,
            lexeme: match ty {
                TokenType::String => source[self.start + 1..self.i - 1].to_string(),
                _ => source[self.start..self.i].to_string(),
            },
        }
    }
}
