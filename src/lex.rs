use anyhow::*;
use std::{collections::HashMap, fs::File, path::Display};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
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
    Nil,
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

pub mod val {
    use anyhow::*;
    use std::fmt;

    use crate::ast::AstWalkError;

    #[derive(Debug, Clone)]
    pub enum ObjectVal {
        Number(f64),
        String(String),
        Boolean(bool),
        Nil,
        // use an Rc for object so we can keep ObjectVal as small as possible
        // OR we can have the language ast::Object wrap a Box/Rc, which is probably more
        // preferrable
        // Obj(Rc<ast::Object>),
    }

    impl ObjectVal {
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
                ObjectVal::Number(_) => "Number".into(),
                ObjectVal::String(_) => "String".into(),
                ObjectVal::Boolean(_) => "Boolean".into(),
                ObjectVal::Nil => "Nil".into(),
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

        pub const fn is_nil(&self) -> bool {
            if let Self::Nil = self {
                true
            } else {
                false
            }
        }
    }

    impl Default for ObjectVal {
        fn default() -> Self {
            Self::Nil
        }
    }

    impl fmt::Display for ObjectVal {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let str = match self {
                ObjectVal::Number(n) => n.to_string(),
                ObjectVal::String(s) => s.clone(),
                ObjectVal::Boolean(b) => b.to_string(),
                ObjectVal::Nil => String::from("nil"),
            };
            write!(f, "{}", str)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub ty: TokenType,
    pub literal: val::ObjectVal,
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

use crate::sys::{is_alpha, is_alpha_numeric, is_digit};

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
    "continue" => TokenType::Continue
};

impl Lexer {
    pub fn scan_tokens_file(filepath: &str) -> anyhow::Result<LexResult> {
        let source = std::fs::read_to_string(filepath)?;
        let s = Self::scan_tokens(source.as_ref());
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
        LexResult {
            tokens: lex.tokens,
            errors: lex.errors,
        }
    }

    #[inline]
    fn advance_cursor(&mut self, n: usize) {
        self.cursor.i += n;
    }

    fn select_number(&mut self) -> Option<val::ObjectVal> {
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
            Some(val::ObjectVal::Number(value))
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

    fn select_string(&mut self) -> Option<val::ObjectVal> {
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
            Some(val::ObjectVal::String(value))
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

    pub fn to_token(&self, source: &str, ty: TokenType, literal: Option<val::ObjectVal>) -> Token {
        Token {
            ty,
            literal: literal.unwrap_or(val::ObjectVal::Nil),
            line: self.lineno,
            lexeme: match ty {
                TokenType::String => source[self.start + 1..self.i - 1].to_string(),
                _ => source[self.start..self.i].to_string(),
            },
        }
    }
}
