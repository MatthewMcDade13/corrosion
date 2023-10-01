use crate::{
    ast::{AstWalkError, AstWalker, Expr, Stmt},
    lex::Cursor,
    value::{Token, TokenType, Value},
};
use anyhow::*;

#[derive(Debug, Clone)]
pub struct Parser {
    cursor: Cursor,
    tokens: Vec<Token>,
}

impl Parser {
    pub fn parse(tokens: &[Token]) -> anyhow::Result<Vec<Stmt>> {
        let mut p = Self {
            cursor: Cursor::new(),
            tokens: tokens.to_vec(),
        };
        let mut statements = Vec::new();
        while !p.is_eof() {
            if let Some(stmt) = p.declaration() {
                statements.push(stmt);
            }
        }
        Ok(statements)
    }

    /// advance cursor to the next expression
    fn synchronize(&mut self) {
        self.advance(1);
        while !self.is_eof() {
            if let TokenType::Semicolon = self.prev().ty {
                break;
            }
            match self.peek().ty {
                TokenType::Struct
                | TokenType::Fn
                | TokenType::Let
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => break,
                _ => self.advance(1),
            }
        }
    }

    // Option as parsing an invalid declaration just results in that declaration getting ignored.
    // we should probably do some logging or error reporting at a higher level so invalid
    // declarations can be known about and arent completely silently ignored.
    fn declaration(&mut self) -> Option<Stmt> {
        let result = if let TokenType::Let = self.peek().ty {
            self.advance(1);
            self.let_statement()
        } else {
            self.statement()
        };
        match result {
            anyhow::Result::Ok(stmt) => Some(stmt),
            Err(_) => {
                self.synchronize();
                None
            }
        }
    }

    fn let_statement(&mut self) -> anyhow::Result<Stmt> {
        if let TokenType::Ident = self.peek().ty {
            let name = self.peek().clone();
            self.advance(1);
            let initializer = if let TokenType::Equal = self.peek().ty {
                self.advance(1);
                Some(self.expression()?)
            } else {
                None
            };

            if let TokenType::Semicolon = self.peek().ty {
                self.advance(1);
                Ok(Stmt::Let { name, initializer })
            } else {
                bail!(
                    "{}",
                    AstWalkError::ParseError {
                        token: self.peek().clone(),
                        message: "Expected ';' after let statement".into()
                    }
                )
            }
        } else {
            bail!(
                "{}",
                AstWalkError::ParseError {
                    token: self.peek().clone(),
                    message: "Expected variable name".into()
                }
            )
        }
    }

    fn statement(&mut self) -> anyhow::Result<Stmt> {
        match self.peek().ty {
            TokenType::Print => {
                self.advance(1);
                self.statement_print()
            }
            TokenType::LeftBrace => {
                self.advance(1);
                Ok(Stmt::Block(self.block()?))
            }
            _ => self.statement_expression(),
        }
    }

    fn block(&mut self) -> anyhow::Result<Vec<Stmt>> {
        let mut statements = Vec::new();
        while self.peek().ty != TokenType::RightBrace && !self.is_eof() {
            match self.declaration() {
                Some(stmt) => statements.push(stmt),
                None => bail!(
                    "{}",
                    AstWalkError::ParseError {
                        token: self.peek().clone(),
                        message: "invalid declaration".into()
                    }
                ),
            };
        }
        if let TokenType::RightBrace = self.peek().ty {
            Ok(statements)
        } else {
            bail!(
                "{}",
                AstWalkError::ParseError {
                    token: self.peek().clone(),
                    message: "Expect '}' after block.".into()
                }
            )
        }
    }

    fn statement_print(&mut self) -> anyhow::Result<Stmt> {
        let expr = self.expression()?;
        if let TokenType::Semicolon = self.peek().ty {
            self.advance(1);
            Ok(Stmt::Print(expr))
        } else {
            bail!(
                "{}",
                AstWalkError::ParseError {
                    token: self.peek().clone(),
                    message: "Expected ';' after value".into()
                }
            )
        }
    }

    fn statement_expression(&mut self) -> anyhow::Result<Stmt> {
        let expr = self.expression()?;
        if let TokenType::Semicolon = self.peek().ty {
            self.advance(1);
            Ok(Stmt::Expression(expr))
        } else {
            bail!(
                "{}",
                AstWalkError::ParseError {
                    token: self.peek().clone(),
                    message: "Expected ';' after expression".into()
                }
            )
        }
    }

    fn assignment(&mut self) -> anyhow::Result<Expr> {
        let expr = self.equality()?;

        if let TokenType::Equal = self.peek().ty {
            self.advance(1);
            let equals = self.prev().clone();
            let value = self.assignment()?;
            if let Expr::Name(name) = expr {
                Ok(Expr::Assignment {
                    name,
                    value: Box::new(value),
                })
            } else {
                bail!(
                    "{}",
                    AstWalkError::ParseError {
                        token: equals,
                        message: "Invalid assignment target".into()
                    }
                )
            }
        } else {
            Ok(expr)
        }
    }

    fn expression(&mut self) -> anyhow::Result<Expr> {
        self.assignment()
    }
    fn term(&mut self) -> anyhow::Result<Expr> {
        // self.expand_binary_expr(ExprRule::Factor, &[TokenType::Minus, TokenType::Plus])
        let mut expr = self.factor()?;
        loop {
            match self.peek().ty {
                TokenType::Minus | TokenType::Plus => {
                    self.advance(1);
                    let operator = self.prev().clone();
                    let right = self.factor()?;
                    expr = Expr::Binary {
                        left: Box::new(expr),
                        operator,
                        right: Box::new(right),
                    }
                }
                _ => break,
            }
        }
        Ok(expr)
    }
    fn comparison(&mut self) -> anyhow::Result<Expr> {
        let mut expr = self.term()?;
        loop {
            match self.peek().ty {
                TokenType::Gt
                | TokenType::Ge
                | TokenType::Lt
                | TokenType::Le
                | TokenType::EqualEqual
                | TokenType::BangEqual => {
                    self.advance(1);
                    let operator = self.prev().clone();
                    let right = self.term()?;
                    expr = Expr::Binary {
                        left: Box::new(expr),
                        operator,
                        right: Box::new(right),
                    }
                }
                _ => break,
            }
        }

        Ok(expr)
    }
    fn factor(&mut self) -> anyhow::Result<Expr> {
        let mut expr = self.unary()?;
        loop {
            match self.peek().ty {
                TokenType::ForwardSlash | TokenType::Star => {
                    self.advance(1);
                    let operator = self.prev().clone();
                    let right = self.unary()?;
                    expr = Expr::Binary {
                        left: Box::new(expr),
                        operator,
                        right: Box::new(right),
                    }
                }
                _ => break,
            }
        }

        Ok(expr)
    }
    fn unary(&mut self) -> anyhow::Result<Expr> {
        match self.peek().ty {
            TokenType::Bang | TokenType::Minus => {
                self.advance(1);
                let operator = self.prev().clone();
                let right = self.unary()?;
                Ok(Expr::Unary {
                    operator,
                    right: Box::new(right),
                })
            }
            _ => self.primary(),
        }
    }
    fn primary(&mut self) -> anyhow::Result<Expr> {
        match self.peek().ty {
            TokenType::False => {
                self.advance(1);
                Ok(Expr::Literal(Value::Boolean(false)))
            }
            TokenType::True => {
                self.advance(1);
                Ok(Expr::Literal(Value::Boolean(true)))
            }
            TokenType::Nil => {
                self.advance(1);
                Ok(Expr::Literal(Value::Nil))
            }
            TokenType::Number | TokenType::String => {
                self.advance(1);
                Ok(Expr::Literal(self.prev().literal.clone()))
            }
            TokenType::LeftParen => {
                self.advance(1);
                let expr = self.expression()?;
                if self.peek().ty == TokenType::RightParen {
                    Ok(Expr::Grouping(Box::new(expr)))
                } else {
                    Err(anyhow!(
                        "Expected matching ending right parenthesis in expression"
                    ))
                }
            }
            TokenType::Ident => {
                let name = self.peek().clone();
                self.advance(1);
                Ok(Expr::Name(name))
            }
            _ => Err(anyhow!(
                "Expected primary or group expression, found: {:?}",
                self.peek()
            )),
        }
    }
    fn equality(&mut self) -> anyhow::Result<Expr> {
        let mut expr = self.comparison()?;
        loop {
            match self.peek().ty {
                TokenType::BangEqual | TokenType::EqualEqual => {
                    self.advance(1);
                    let operator = self.prev().clone();
                    let right = self.comparison()?;
                    expr = Expr::Binary {
                        left: Box::new(expr),
                        operator,
                        right: Box::new(right),
                    }
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn is_eof(&self) -> bool {
        if let TokenType::Eof = self.peek().ty {
            true
        } else {
            false
        }
    }

    fn advance(&mut self, n: usize) {
        assert!(!self.is_eof(), "Tried to advance cursor at EOF token");
        assert!(
            self.cursor.i + n < self.tokens.len(),
            "advancing cursor past end of tokens"
        );
        self.cursor.i += n;
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.cursor.i]
    }

    fn prev(&self) -> &Token {
        &self.tokens[self.cursor.i - 1]
    }
}
