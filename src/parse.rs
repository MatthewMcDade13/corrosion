use crate::{
    ast::expr::{self, Expr, ExprRule},
    lex::{val::ObjectVal, Cursor, Token, TokenType},
};
use anyhow::*;

#[derive(Debug, Clone)]
pub struct Parser {
    cursor: Cursor,
    tokens: Vec<Token>,
}

impl Parser {
    pub fn parse(tokens: &[Token]) -> anyhow::Result<Expr> {
        let mut p = Self {
            cursor: Cursor::new(),
            tokens: tokens.to_vec(),
        };
        let expr = p.expression()?;

        Ok(expr)
    }

    fn expression(&mut self) -> anyhow::Result<Expr> {
        self.equality()
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
                Ok(Expr::Literal(ObjectVal::Boolean(false)))
            }
            TokenType::True => {
                self.advance(1);
                Ok(Expr::Literal(ObjectVal::Boolean(true)))
            }
            TokenType::Unit => {
                self.advance(1);
                Ok(Expr::Literal(ObjectVal::Unit))
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
            _ => Err(anyhow!(
                "Expected primary or group expression, found: {:?}",
                self.peek().ty
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

    fn expand_binary_expr(
        &mut self,
        expr_type: expr::ExprRule,
        match_tokens: &[TokenType],
    ) -> Result<Expr> {
        let mut expr = self.select_expand_expr(expr_type)?;
        loop {
            if match_tokens.iter().any(|x| self.peek().ty == *x) {
                self.advance(1);
                let operator = self.prev().clone();
                let right = self.select_expand_expr(expr_type)?;
                expr = Expr::Binary {
                    left: Box::new(expr),
                    operator,
                    right: Box::new(right),
                }
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn select_expand_expr(&mut self, expr_type: expr::ExprRule) -> anyhow::Result<Expr> {
        match expr_type {
            expr::ExprRule::Expression => self.expression(),
            expr::ExprRule::Equality => self.equality(),
            expr::ExprRule::Comparison => self.comparison(),
            expr::ExprRule::Term => self.term(),
            expr::ExprRule::Factor => self.factor(),
            expr::ExprRule::Unary => self.unary(),
            expr::ExprRule::Primary => self.primary(),
        }
    }
}
