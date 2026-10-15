use anyhow::{Result, anyhow};

use crate::lox::{
    Lox,
    ast::Expr,
    token::{Token, TokenType},
};

pub(crate) struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Expr> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Expr> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr> {
        let mut expr = self.comparison()?;
        while self.match_(|t| matches!(t, TokenType::EqualEqual | TokenType::BangEqual)) {
            let operator = self.previous().ty;
            let right = self.term()?;
            expr = Expr::binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr> {
        let mut expr = self.term()?;
        while self.match_(|t| {
            matches!(
                t,
                TokenType::Greater
                    | TokenType::GreaterEqual
                    | TokenType::Less
                    | TokenType::LessEqual
            )
        }) {
            let operator = self.previous().ty;
            let right = self.term()?;
            expr = Expr::binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr> {
        let mut expr = self.factor()?;
        while self.match_(|t| matches!(t, TokenType::Minus | TokenType::Plus)) {
            let operator = self.previous().ty;
            let right = self.factor()?;
            expr = Expr::binary(expr, operator, right)
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr> {
        let mut expr = self.unary()?;
        while self.match_(|t| matches!(t, TokenType::Slash | TokenType::Star)) {
            let operator = self.previous().ty;
            let right = self.unary()?;
            expr = Expr::binary(expr, operator, right)
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr> {
        if self.match_(|t| matches!(t, TokenType::Bang | TokenType::Minus)) {
            let operator = self.previous().ty;
            let right = self.unary()?;
            return Ok(Expr::unary(operator, right));
        }
        return self.primary();
    }

    fn primary(&mut self) -> Result<Expr> {
        if self.match_(|t| {
            matches!(
                t,
                TokenType::False
                    | TokenType::True
                    | TokenType::Nil
                    | TokenType::Num(_)
                    | TokenType::Str(_)
            )
        }) {
            return Ok(Expr::Literal(self.previous().ty));
        }

        if self.match_(|t| matches!(t, TokenType::LeftParen)) {
            let expr = self.expression()?;
            self.consume(&TokenType::RightParen, "Expect ')' after expression.")?;
            return Ok(Expr::grouping(expr));
        }

        Err(anyhow!(Lox::error(self.peek(), "Expect expression.")))
    }

    // --- Helper Methods

    fn match_(&mut self, pred: impl Fn(&TokenType) -> bool) -> bool {
        if !self.is_eof() && pred(&self.peek().ty) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume(&mut self, cond: &TokenType, err_msg: &str) -> Result<Option<Token>> {
        if self.check(cond) {
            return Ok(self.advance());
        }

        Err(anyhow!(Lox::error(self.peek(), err_msg)))
    }

    fn check(&self, ty: &TokenType) -> bool {
        !self.is_eof() && ty == &self.peek().ty
    }

    fn advance(&mut self) -> Option<Token> {
        (!self.is_eof()).then(|| {
            self.current += 1;
            self.previous()
        })
    }

    fn previous(&self) -> Token {
        debug_assert!(self.current > 0);
        self.tokens.get(self.current - 1).cloned().unwrap()
    }

    fn peek(&self) -> Token {
        debug_assert!(self.current < self.tokens.len());
        self.tokens.get(self.current).cloned().unwrap()
    }

    fn is_eof(&self) -> bool {
        matches!(self.peek().ty, TokenType::EOF)
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_eof() {
            match self.peek().ty {
                TokenType::Semicolon
                | TokenType::Class
                | TokenType::For
                | TokenType::Fun
                | TokenType::If
                | TokenType::Print
                | TokenType::Return
                | TokenType::Var
                | TokenType::While => break,
                _ => {
                    self.advance();
                }
            }
        }
    }
}
