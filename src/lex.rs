use std::{collections::HashMap, fs::File};

#[derive(Debug, Clone, Copy)]
pub enum TokenType {
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
}

#[derive(Debug, Clone)]
pub enum Literal {
    Number(f64),
    Str(String),
    Boolean(bool),
    Nil,
    Unknown(String),
}

impl Default for Literal {
    fn default() -> Self {
        Self::Unknown("unassigned".into())
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    ty: TokenType,
    literal: Literal,
    line: u32,
    lexeme: String,
}

impl ToString for Token {
    fn to_string(&self) -> String {
        let Self {
            ty,
            literal,
            lexeme,
            line,
        } = self;
        format!("LineNo:{line} {ty:?} :: {lexeme} :: {literal:?}")
    }
}
#[derive(Debug, Clone)]
pub struct Lexer {
    source: Vec<u8>,
    cursor: Cursor,
    tokens: Vec<Token>,
    errors: Vec<String>,
}
/*
*
    And,

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
* */
use phf::phf_map;

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
    pub fn scan_tokens(source_str: &str) -> Vec<Token> {
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
                            lex.cursor.i += 1;
                        }
                        TokenType::Comment
                    } else {
                        TokenType::ForwardSlash
                    };
                    (ty, None)
                }
                _ => {
                    todo!()
                }
            };
            let token = lex.cursor.to_token(source_str, ty, literal);
            lex.tokens.push(token);
        }
        lex.tokens
    }

    fn is_cursor_at_end(&self) -> bool {
        self.cursor.i >= self.source.len()
    }

    fn match_next(&mut self, c: char) -> bool {
        if self.is_cursor_at_end() || self.peek() != c {
            false
        } else {
            self.cursor.i += 1;
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
                self.cursor.i += 1;
            } else {
                self.cursor.start = self.cursor.i;
                self.cursor.i += 1;
                return c;
            }
        }
        '\0'
    }
    fn new(source_str: &str) -> Self {
        Self {
            source: source_str.as_bytes().to_vec(),
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

#[derive(Debug, Default, Clone, Copy)]
pub struct Cursor {
    start: usize,
    i: usize,
    lineno: u32,
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            lineno: 1,
            ..Self::default()
        }
    }

    pub fn to_token(&self, source: &str, ty: TokenType, literal: Option<Literal>) -> Token {
        Token {
            ty,
            literal: literal.unwrap_or(Literal::Nil),
            line: self.lineno,
            lexeme: source[self.start..self.i].to_string(),
        }
    }
}
