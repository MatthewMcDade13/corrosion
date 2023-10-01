use std::ops::Add;

use anyhow::bail;
use log::debug;

use crate::{
    lex::Lexer,
    value::{Token, TokenType, Value},
    vm::{Opcode, OpcodeType, VM},
};

struct Parser {
    tokens: Vec<Token>,
    i: usize,
    bytecode: Chunk,
}

impl Parser {
    #[inline]
    fn advance(&mut self, n: usize) {
        self.i += n;
    }
    fn expression(&mut self) -> anyhow::Result<()> {
        self.parse_precedence(Precedence::Assignment)
    }

    fn string(&mut self) -> anyhow::Result<()> {
        self.bytecode.add_constant(self.prev().literal.clone());
        Ok(())
    }

    // true, false, nil
    fn literal(&mut self) -> anyhow::Result<()> {
        match self.prev().ty {
            TokenType::Nil => self.bytecode.add_opcode(OpcodeType::Nil.into()),
            TokenType::False => self.bytecode.add_opcode(OpcodeType::False.into()),
            TokenType::True => self.bytecode.add_opcode(OpcodeType::True.into()),
            _ => unreachable!(
                "Expected token to be a literal (true, false, nil), got: {}",
                self.prev()
            ),
        };
        Ok(())
    }

    fn number(&mut self) -> anyhow::Result<()> {
        // TODO :: Typecheck this!!!
        if self.prev().ty == TokenType::Number {
            self.bytecode.add_constant(self.prev().literal.clone());
            Ok(())
        } else {
            bail!("Expected number, got {}", self.prev().literal.clone());
        }
    }

    fn grouping(&mut self) -> anyhow::Result<()> {
        self.expression()?;
        self.expect(TokenType::RightParen, "Expected ')' at end of grouping")
    }

    fn unary(&mut self) -> anyhow::Result<()> {
        let op_type = self.prev().ty;

        self.expression()?;

        match op_type {
            TokenType::Minus => self.bytecode.add_opcode(OpcodeType::Negate.into()),
            TokenType::Bang => self.bytecode.add_opcode(OpcodeType::Not.into()),
            _ => unreachable!("Unreachable branch in compiler::Parser::unary"),
        };
        Ok(())
    }

    fn binary(&mut self) -> anyhow::Result<()> {
        let op_type = self.prev().ty;
        let rule = get_parse_rule(op_type);

        self.parse_precedence(rule.precedence.next())?;
        match op_type {
            TokenType::Plus => self.bytecode.add_opcode(OpcodeType::Add.into()),
            TokenType::Minus => self.bytecode.add_opcode(OpcodeType::Subtract.into()),
            TokenType::Star => self.bytecode.add_opcode(OpcodeType::Mult.into()),
            TokenType::ForwardSlash => self.bytecode.add_opcode(OpcodeType::Div.into()),
            TokenType::BangEqual => self
                .bytecode
                .add_opcodes(OpcodeType::Equal.into(), OpcodeType::Not.into()),
            TokenType::EqualEqual => self.bytecode.add_opcode(OpcodeType::Equal.into()),
            TokenType::Gt => self.bytecode.add_opcode(OpcodeType::GreaterThan.into()),
            TokenType::Ge => self
                .bytecode
                .add_opcodes(OpcodeType::LessThan.into(), OpcodeType::Not.into()),
            TokenType::Lt => self.bytecode.add_opcode(OpcodeType::LessThan.into()),
            TokenType::Le => self
                .bytecode
                .add_opcodes(OpcodeType::GreaterThan.into(), OpcodeType::Not.into()),
            _ => unreachable!("Unreachable branch in compiler::Parser::binary"),
        };
        Ok(())
    }

    fn expect(&mut self, ty: TokenType, message: &str) -> anyhow::Result<()> {
        if self.current().ty == ty {
            self.advance(1);
            Ok(())
        } else {
            bail!(message.to_string());
        }
    }

    fn current(&self) -> &Token {
        &self.tokens[self.i]
    }

    fn prev(&self) -> &Token {
        &self.tokens[self.i - 1]
    }

    fn parse_precedence(&mut self, precedence: Precedence) -> anyhow::Result<()> {
        self.advance(1);
        if let Some(prefix) = get_parse_rule(self.prev().ty).prefix {
            prefix(self)?;
        } // else {
          //             // bail!("Expected expression");
          // }

        while precedence <= get_parse_rule(self.current().ty).precedence {
            self.advance(1);
            if let Some(infix) = get_parse_rule(self.prev().ty).infix {
                infix(self)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Precedence {
    None = 0,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
    PrecedenceCount,
}

impl Precedence {
    const fn next(&self) -> Self {
        match self {
            Precedence::None => Precedence::Assignment,
            Precedence::Assignment => Precedence::Or,
            Precedence::Or => Precedence::And,
            Precedence::And => Precedence::Equality,
            Precedence::Equality => Precedence::Comparison,
            Precedence::Comparison => Precedence::Term,
            Precedence::Term => Precedence::Factor,
            Precedence::Factor => Precedence::Unary,
            Precedence::Unary => Precedence::Call,
            Precedence::Call => Precedence::Primary,
            Precedence::Primary => Precedence::PrecedenceCount,
            _ => Precedence::None,
        }
    }
}

type ParseFn = fn(&mut Parser) -> anyhow::Result<()>;

#[derive(Debug, Clone)]
struct ParseRule {
    prefix: Option<ParseFn>,
    infix: Option<ParseFn>,
    precedence: Precedence,
}

impl ParseRule {
    pub fn new(prefix: Option<ParseFn>, infix: Option<ParseFn>, precedence: Precedence) -> Self {
        Self {
            prefix,
            infix,
            precedence,
        }
    }

    pub fn with_prefix(prefix: ParseFn, precedence: Option<Precedence>) -> Self {
        Self {
            prefix: Some(prefix),
            infix: None,
            precedence: match precedence {
                Some(p) => p,
                None => Precedence::None,
            },
        }
    }

    pub fn with_infix(infix: ParseFn, precedence: Option<Precedence>) -> Self {
        Self {
            prefix: None,
            infix: Some(infix),
            precedence: precedence.unwrap_or(Precedence::None),
        }
    }

    pub fn none() -> Self {
        Self {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        }
    }
}

/*
*
pub enum TokenType {





    TokenTypeCount,
}
* */

fn get_parse_rule(ty: TokenType) -> ParseRule {
    match ty {
        TokenType::Print => ParseRule::none(),
        TokenType::FatArrow => ParseRule::none(),
        TokenType::LeftBrace => ParseRule::none(),
        TokenType::RightBrace => ParseRule::none(),
        TokenType::LeftParen => ParseRule::with_prefix(Parser::grouping, None),
        TokenType::RightParen => ParseRule::none(),
        TokenType::Comma => ParseRule::none(),
        TokenType::Dot => ParseRule::none(),
        TokenType::Minus => ParseRule {
            prefix: Some(Parser::unary),
            infix: Some(Parser::binary),
            precedence: Precedence::Term,
        },
        TokenType::Plus => ParseRule::with_infix(Parser::binary, Some(Precedence::Term)),
        TokenType::Semicolon => ParseRule::none(),
        TokenType::ForwardSlash => ParseRule::with_infix(Parser::binary, Some(Precedence::Factor)),
        TokenType::Star => ParseRule::with_infix(Parser::binary, Some(Precedence::Factor)),
        TokenType::Bang => ParseRule::with_prefix(Parser::unary, None),
        TokenType::Equal => ParseRule::none(),
        TokenType::BangEqual => ParseRule::with_infix(Parser::binary, Some(Precedence::Equality)),
        TokenType::EqualEqual => ParseRule::with_infix(Parser::binary, Some(Precedence::Equality)),
        TokenType::Gt => ParseRule::with_infix(Parser::binary, Some(Precedence::Comparison)),
        TokenType::Ge => ParseRule::with_infix(Parser::binary, Some(Precedence::Comparison)),
        TokenType::Lt => ParseRule::with_infix(Parser::binary, Some(Precedence::Comparison)),
        TokenType::Le => ParseRule::with_infix(Parser::binary, Some(Precedence::Comparison)),
        TokenType::Ident => ParseRule::none(),
        TokenType::String => ParseRule::with_prefix(Parser::string, None),
        TokenType::Number => ParseRule::with_prefix(Parser::number, None),
        TokenType::And => ParseRule::none(),
        TokenType::Struct => ParseRule::none(),
        TokenType::Trait => ParseRule::none(),
        TokenType::Impl => ParseRule::none(),
        TokenType::Else => ParseRule::none(),
        TokenType::False => ParseRule::with_prefix(Parser::literal, None),
        TokenType::True => ParseRule::with_prefix(Parser::literal, None),
        TokenType::Fn => ParseRule::none(),
        TokenType::If => ParseRule::none(),
        TokenType::Nil => ParseRule::with_prefix(Parser::literal, None),
        TokenType::Or => ParseRule::none(),
        TokenType::Return => ParseRule::none(),
        TokenType::Super => ParseRule::none(),
        TokenType::ThisSelf => ParseRule::none(),
        TokenType::Let => ParseRule::none(),
        TokenType::Const => ParseRule::none(),
        TokenType::Eof => ParseRule::none(),
        TokenType::Loop => ParseRule::none(),
        TokenType::For => ParseRule::none(),
        TokenType::While => ParseRule::none(),
        TokenType::Break => ParseRule::none(),
        TokenType::Switch => ParseRule::none(),
        TokenType::Continue => ParseRule::none(),
        TokenType::Comment => ParseRule::none(),
        TokenType::Unknown => ParseRule::none(),
        TokenType::Colon => ParseRule::none(),
        TokenType::DoubleColon => ParseRule::none(),
    }
}

#[derive(Debug, Clone, Default)]
pub struct Chunk {
    instructions: Vec<Opcode>,
    constants: Vec<Value>,
    lines: Vec<u64>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            instructions: Vec::with_capacity(8),
            constants: Vec::with_capacity(8),
            lines: Vec::with_capacity(8),
        }
    }

    #[inline]
    pub fn instructions_len(&self) -> usize {
        self.instructions.len()
    }

    #[inline]
    pub fn constant_at(&self, index: usize) -> &Value {
        &self.constants[index]
    }

    #[inline]
    pub fn opcode_at(&self, index: usize) -> Opcode {
        self.instructions[index]
    }

    #[inline]
    pub fn add_opcode(&mut self, code: Opcode) {
        self.instructions.push(code);
    }

    #[inline]
    pub fn add_opcodes(&mut self, a: Opcode, b: Opcode) {
        self.instructions.push(a);
        self.instructions.push(b);
    }

    pub fn add_constant(&mut self, v: Value) {
        self.constants.push(v);
        let cindex = self.constants.len() - 1;
        self.add_opcode(Opcode(OpcodeType::Constant as usize));
        self.add_opcode(Opcode(cindex));
    }

    pub fn print_instructions(&self) -> String {
        format!("{:?}", self.instructions)
    }

    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);
        let mut i = 0;
        while i < self.instructions.len() {
            print!("{:04} ", i);
            let op = self.opcode_at(i);
            i += 1;
            match op.ty() {
                OpcodeType::Return => {
                    println!("Opcode::Return");
                }
                OpcodeType::Constant => {
                    let cindex = self.opcode_at(i);
                    let constant = &self.constants[cindex.0];
                    i += 1;
                    println!("Opcode::Constant {constant}");
                }
                OpcodeType::Negate => {
                    println!("Opcode::Negate");
                }
                _ => {
                    println!("Unknown opcode :: {}", op.0);
                }
            }
        }
    }
}

pub fn compile_source(source: &str) -> anyhow::Result<Chunk> {
    let result = Lexer::scan_tokens(source);
    if result.errors.len() == 0 {
        compile(&result.tokens)
    } else {
        bail!("LEX ERROR(S): {:?}", result.errors)
    }
}

pub fn compile(tokens: &[Token]) -> anyhow::Result<Chunk> {
    let mut p = Parser {
        tokens: tokens.to_vec(),
        i: 0,
        bytecode: Chunk::new(),
    };

    // p.advance(1);
    p.expression()?;
    // p.expect(
    // TokenType::Eof,
    // &format!("Expected end of file token, got {:?}", p.current().ty),
    // )?;
    p.bytecode.add_opcode(OpcodeType::Return.into());
    Ok(p.bytecode)
}
