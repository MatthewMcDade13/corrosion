use std::ops::Add;

use anyhow::bail;
use log::debug;

use crate::{
    lex::Lexer,
    value::{Object, Token, TokenType, Value},
    vm::{Opcode, OpcodeType, VM},
};

#[derive(Debug, Clone)]
struct Parser {
    tokens: Vec<Token>,
    i: usize,
    bytecode: Chunk,
    compiler: Compiler,
}

impl Parser {
    #[inline]
    fn advance(&mut self, n: usize) {
        self.i += n;
    }

    fn declaration(&mut self) -> anyhow::Result<()> {
        let result = if let TokenType::Let = self.current().ty {
            self.advance(1);
            self.let_declaration()
        } else {
            self.statement()
        };

        match result {
            Ok(_) => Ok(()),
            Err(err) => {
                println!("Compiler::Parser => [ERROR]: {}", err);
                self.synchronize()
            }
        }
    }

    fn let_declaration(&mut self) -> anyhow::Result<()> {
        let global = self.parse_variable()?;

        if let TokenType::Equal = self.current().ty {
            self.advance(1);
            self.expression()?;
        } else {
            self.bytecode.add_opcode(OpcodeType::Nil.into());
        };
        self.expect(
            TokenType::Semicolon,
            "let_declaration :: Expected ';' after let declaration",
        )?;
        self.define_variable(global);
        Ok(())
    }

    fn statement(&mut self) -> anyhow::Result<()> {
        match self.current().ty {
            TokenType::Print => {
                self.advance(1);
                self.print_statement()
            }
            TokenType::LeftBrace => {
                self.advance(1);
                self.begin_scope();

                self.block()?;
                self.end_scope();
                Ok(())
            }
            _ => self.expression_statement(),
        }
    }

    fn expression(&mut self) -> anyhow::Result<()> {
        self.parse_precedence(Precedence::Assignment)
    }

    fn block(&mut self) -> anyhow::Result<()> {
        while self.current().ty != TokenType::RightBrace && self.current().ty != TokenType::Eof {
            self.declaration()?;
        }
        self.expect(TokenType::RightBrace, "Expected '}' after block statement")
    }

    fn expression_statement(&mut self) -> anyhow::Result<()> {
        self.expression()?;
        self.expect(
            TokenType::Semicolon,
            "expression_statement :: Expected ';' at end of statement",
        )?;
        self.bytecode.add_opcode(OpcodeType::Pop.into());
        Ok(())
    }

    fn print_statement(&mut self) -> anyhow::Result<()> {
        self.expression()?;
        self.expect(
            TokenType::Semicolon,
            "print_statement :: Expected ';' at end of statement",
        )?;
        self.bytecode.add_opcode(OpcodeType::Print.into());
        Ok(())
    }

    fn define_variable(&mut self, global_index: usize) {
        if self.compiler.scope_depth > 0 {
            // self.compiler.initialize_local(0);
            return ();
        }
        self.bytecode
            .add_opcodes(OpcodeType::DefineGlobal.into(), global_index.into());
    }

    fn declare_variable(&mut self) -> anyhow::Result<()> {
        if self.compiler.scope_depth == 0 {
            Ok(())
        } else {
            let name = self.prev().clone();
            self.compiler.push_local(name);
            Ok(())
        }
    }

    fn string(&mut self, _: bool) -> anyhow::Result<()> {
        self.bytecode.add_constant(self.prev().literal.clone());
        Ok(())
    }

    fn parse_variable(&mut self) -> anyhow::Result<usize> {
        self.expect(TokenType::Ident, "Expected name for let declaration")?;

        self.declare_variable()?;
        if self.compiler.scope_depth > 0 {
            return Ok(0);
        }
        let prev_tok = self.prev().clone();
        let name_index = self.bytecode.add_constant_ident(&prev_tok);
        Ok(name_index)
    }

    fn synchronize(&mut self) -> anyhow::Result<()> {
        while self.current().ty != TokenType::Eof {
            if let TokenType::Semicolon = self.prev().ty {
                return Ok(());
            } else {
                match self.current().ty {
                    TokenType::Struct
                    | TokenType::Fn
                    | TokenType::Let
                    | TokenType::For
                    | TokenType::If
                    | TokenType::While
                    | TokenType::Print
                    | TokenType::Return => return Ok(()),
                    _ => self.advance(1),
                };
            };
        }
        Ok(())
    }

    fn variable(&mut self, can_assign: bool) -> anyhow::Result<()> {
        self.named_variable(&self.prev().clone(), can_assign)
    }

    fn named_variable(&mut self, name: &Token, can_assign: bool) -> anyhow::Result<()> {
        // let arg = self.bytecode.add_constant_ident(token);
        let (get, set, arg) = {
            if let Some(i) = self.compiler.resolve_local(name) {
                (
                    Opcode::from(OpcodeType::GetLocal),
                    Opcode::from(OpcodeType::SetLocal),
                    i,
                )
            } else {
                (
                    Opcode::from(OpcodeType::GetGlobal),
                    Opcode::from(OpcodeType::SetGlobal),
                    self.bytecode.add_constant_ident(name),
                )
                // (Opcode::from(OpcodeType::GetGlobal, Opcode::from(OpcodeType::SetGlobal))
            }
        };
        // println!("named_variable => locals: {:?}", self.compiler.locals);
        // println!("named_variable => get: {}, set: {}, arg: {}", get, set, arg);
        if can_assign && self.current().ty == TokenType::Equal {
            self.advance(1);
            self.expression()?;
            self.bytecode.add_opcodes(set, arg.into());
        } else {
            self.bytecode.add_opcodes(get, arg.into());
        };

        Ok(())
    }

    // true, false, nil
    fn literal(&mut self, _: bool) -> anyhow::Result<()> {
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

    fn number(&mut self, _: bool) -> anyhow::Result<()> {
        // TODO :: Typecheck this!!!
        if self.prev().ty == TokenType::Number {
            self.bytecode.add_constant(self.prev().literal.clone());
            Ok(())
        } else {
            bail!("Expected number, got {}", self.prev().literal.clone());
        }
    }

    fn grouping(&mut self, _: bool) -> anyhow::Result<()> {
        self.expression()?;
        self.expect(TokenType::RightParen, "Expected ')' at end of grouping")
    }

    fn unary(&mut self, _: bool) -> anyhow::Result<()> {
        let op_type = self.prev().ty;

        self.expression()?;

        match op_type {
            TokenType::Minus => self.bytecode.add_opcode(OpcodeType::Negate.into()),
            TokenType::Bang => self.bytecode.add_opcode(OpcodeType::Not.into()),
            _ => unreachable!("Unreachable branch in compiler::Parser::unary"),
        };
        Ok(())
    }

    fn binary(&mut self, _: bool) -> anyhow::Result<()> {
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
            bail!(
                "Compiler::Parser => {}; got: {}",
                message.to_string(),
                self.current()
            );
        }
    }

    #[inline]
    fn current(&self) -> &Token {
        &self.tokens[self.i]
    }

    #[inline]
    fn prev(&self) -> &Token {
        &self.tokens[self.i - 1]
    }

    fn begin_scope(&mut self) {
        self.compiler.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.compiler.scope_depth -= 1;
        while self.compiler.locals.len() > 0 {
            let back = self.compiler.locals_top();

            if back.depth > self.compiler.scope_depth {
                let _ = self.compiler.pop_local();
                self.bytecode.add_opcode(OpcodeType::Pop.into());
            } else {
                break;
            }
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) -> anyhow::Result<()> {
        self.advance(1);
        let can_assign = precedence <= Precedence::Assignment;
        if let Some(prefix) = get_parse_rule(self.prev().ty).prefix {
            prefix(self, can_assign)?;
        } // else {
          //             // bail!("Expected expression");
          // }

        while precedence <= get_parse_rule(self.current().ty).precedence {
            self.advance(1);
            if let Some(infix) = get_parse_rule(self.prev().ty).infix {
                infix(self, can_assign)?;
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

type ParseFn = fn(&mut Parser, bool) -> anyhow::Result<()>;

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
        TokenType::Ident => ParseRule::with_prefix(Parser::variable, None),
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

    pub fn add_constant(&mut self, v: Value) -> usize {
        let cindex = self.push_constant(v);
        self.add_opcode(Opcode(OpcodeType::Constant as usize));
        self.add_opcode(Opcode(cindex));
        cindex
    }

    pub fn add_constant_ident(&mut self, token: &Token) -> usize {
        // self.add_constant(Value::Obj(Object::String(token.lexeme.clone())))
        let ident = Value::Obj(Object::String(token.lexeme.clone()));
        self.push_constant(ident)
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
                OpcodeType::Unknown => {}
                _ => {}
            }
        }
    }

    fn push_constant(&mut self, v: Value) -> usize {
        self.constants.push(v);
        self.constants.len() - 1
    }
}

#[derive(Debug, Clone)]
struct Local {
    name: Token,
    depth: usize,
}

#[derive(Debug, Clone)]
pub struct Compiler {
    locals: Vec<Local>,
    scope_depth: usize,
}

impl Compiler {
    fn new() -> Self {
        Self {
            locals: Vec::with_capacity(256),
            scope_depth: 0,
        }
    }

    fn resolve_local(&self, name: &Token) -> Option<usize> {
        let mut i = self.locals.len() as isize - 1;
        while i >= 0 {
            let l = &self.locals[i as usize];

            if name.lexeme == l.name.lexeme {
                return Some(i as usize);
            }
            i -= 1;
        }
        None
    }

    fn push_local(&mut self, token: Token) {
        self.locals.push(Local {
            name: token,
            depth: self.scope_depth,
        });
    }

    fn locals_top(&self) -> &Local {
        self.locals
            .last()
            .expect("Cannot get top of Compiler::locals; array empty.")
    }

    fn pop_local(&mut self) -> Option<Local> {
        self.locals.pop()
    }

    pub fn compile_source(source: &str) -> anyhow::Result<Chunk> {
        let result = Lexer::scan_tokens(source.trim());
        if result.errors.len() == 0 {
            Self::compile(&result.tokens)
        } else {
            bail!("LEX ERROR(S): {:?}", result.errors)
        }
    }

    pub fn compile(tokens: &[Token]) -> anyhow::Result<Chunk> {
        let mut p = Parser {
            tokens: tokens.to_vec(),
            i: 0,
            bytecode: Chunk::new(),
            compiler: Compiler::new(),
        };

        // p.advance(1);
        while p.current().ty != TokenType::Eof {
            p.declaration()?;
        }
        // p.expect(
        // TokenType::Eof,
        // &format!("Expected end of file token, got {:?}", p.current().ty),
        // )?;
        p.bytecode.add_opcode(OpcodeType::Return.into());
        Ok(p.bytecode)
    }
}
