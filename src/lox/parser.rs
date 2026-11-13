use anyhow::{Result, bail};

use crate::lox::{
    Lox,
    ast::{Expr, Stmt},
    token::{Token, TokenType},
};

pub struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>> {
        let mut stmts = vec![];
        while !self.is_eof() {
            stmts.push(self.statement()?);
        }
        Ok(stmts)
    }

    fn statement(&mut self) -> Result<Stmt> {
        if self.match_(|t| matches!(t, TokenType::Print)) {
            self.print_statement()
        } else if self.match_(|t| matches!(t, TokenType::Var)) {
            self.variable_declaration()
        } else if self.match_(|t| matches!(t, TokenType::Fun)) {
            self.function_declaration("function")
        } else if self.match_(|t| matches!(t, TokenType::LeftBrace)) {
            self.block_statement()
        } else if self.match_(|t| matches!(t, TokenType::If)) {
            self.if_statement()
        } else if self.match_(|t| matches!(t, TokenType::While)) {
            self.while_statement()
        } else if self.match_(|t| matches!(t, TokenType::For)) {
            self.for_statement()
        } else {
            self.expression_statement()
        }
    }

    fn print_statement(&mut self) -> Result<Stmt> {
        let val = self.expression()?;
        self.consume(&TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print(val))
    }

    fn expression_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        self.consume(&TokenType::Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expr(expr))
    }

    fn if_statement(&mut self) -> Result<Stmt> {
        self.consume(&TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(&TokenType::RightParen, "Expect ')' after if condition.")?;

        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.match_(|t| matches!(t, TokenType::Else)) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn while_statement(&mut self) -> Result<Stmt> {
        self.consume(&TokenType::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(&TokenType::RightParen, "Expect ')' after while condition")?;

        let body = self.statement()?;

        Ok(Stmt::While {
            condition,
            body: Box::new(body),
        })
    }

    fn for_statement(&mut self) -> Result<Stmt> {
        self.consume(&TokenType::LeftParen, "Expect '(' after 'for'.")?;

        let initializer = if self.match_(|t| matches!(t, TokenType::Semicolon)) {
            None
        } else if self.match_(|t| matches!(t, TokenType::Var)) {
            Some(self.variable_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if !self.check(&TokenType::RightParen) {
            self.expression()?
        } else {
            // An omitted condition will be 'true'
            Expr::Literal(TokenType::True)
        };
        self.consume(&TokenType::Semicolon, "Expect ';' after loop condition")?;

        let increment = if !self.check(&TokenType::RightParen) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(&TokenType::RightParen, "Expect ')' after for clauses")?;

        let mut body = self.statement()?;

        if let Some(inc) = increment {
            body = Stmt::Block(vec![body, Stmt::Expr(inc)]);
        }

        body = Stmt::While {
            condition,
            body: Box::new(body),
        };

        if let Some(init) = initializer {
            body = Stmt::Block(vec![init, body]);
        }

        Ok(body)
    }

    fn block_statement(&mut self) -> Result<Stmt> {
        let mut statements = vec![];

        while !self.check(&TokenType::RightBrace) {
            statements.push(self.statement()?);
        }

        self.consume(&TokenType::RightBrace, "Expect '}' after block.")?;

        Ok(Stmt::Block(statements))
    }

    fn variable_declaration(&mut self) -> Result<Stmt> {
        let identifier = match self.advance() {
            Some(Token { ty, line: _ }) => {
                debug_assert!(matches!(ty, TokenType::Ident(_)));
                ty
            }
            _ => bail!("Expect identifier in variable declaration"),
        };

        let initializer = match self.advance() {
            Some(Token {
                ty: TokenType::Equal,
                line: _,
            }) => {
                let expr = Some(self.expression()?);
                self.consume(
                    &TokenType::Semicolon,
                    "Expect ';' after variable declaration",
                )?;
                expr
            }
            Some(Token {
                ty: TokenType::Semicolon,
                line: _,
            }) => None,
            _ => bail!("Invalid variable declaration"),
        };

        Ok(Stmt::Var {
            identifier,
            initializer,
        })
    }

    fn function_declaration(&mut self, kind: &str) -> Result<Stmt> {
        let name = self
            .consume(&TokenType::Ident("placeholder".to_string()), "HELP!!!")?
            .expect("unreachable")
            .ty;

        self.consume(&TokenType::LeftParen, &format!("Expect {kind} name"))?;

        let mut params = vec![];
        if !self.check(&TokenType::RightParen) {
            self.add_param(&mut params)?;
            while self.match_(|t| matches!(t, TokenType::Comma)) {
                self.add_param(&mut params)?;
            }
        }
        self.consume(&TokenType::RightParen, "Expect ')' after parameters.")?;
        self.consume(
            &TokenType::LeftBrace,
            &format!("Expect '{{' before {kind} body."),
        )?;

        let body = self.block_statement()?;
        debug_assert!(matches!(body, Stmt::Block(_)));

        Ok(Stmt::FunctionDef {
            name,
            params,
            body: Box::new(body),
        })
    }

    fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
        let mut expr = self.logical_or()?;

        if self.match_(|t| matches!(t, TokenType::Equal)) {
            let equals = self.previous();
            let value = self.assignment()?;

            match &expr {
                Expr::Variable(tt @ TokenType::Ident(_)) => {
                    expr = Expr::assign(tt.to_owned(), value)
                }
                _ => bail!(Lox::error(equals, "Invalid assignment target.")),
            }
        }

        return Ok(expr);
    }

    fn logical_or(&mut self) -> Result<Expr> {
        let mut expr = self.logical_and()?;
        while self.match_(|t| matches!(t, TokenType::Or)) {
            let operator = self.previous().ty;
            let right = self.logical_and()?;
            expr = Expr::logical(expr, operator, right);
        }

        Ok(expr)
    }

    fn logical_and(&mut self) -> Result<Expr> {
        let mut expr = self.equality()?;
        while self.match_(|t| matches!(t, TokenType::And)) {
            let operator = self.previous().ty;
            let right = self.equality()?;
            expr = Expr::logical(expr, operator, right);
        }

        Ok(expr)
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
        self.function_call()
    }

    fn function_call(&mut self) -> Result<Expr> {
        let mut expr = self.primary()?;
        loop {
            if self.match_(|t| matches!(t, TokenType::LeftParen)) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
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

        if self.match_(|t| matches!(t, TokenType::Ident(_))) {
            return Ok(Expr::Variable(self.previous().ty));
        }

        if self.match_(|t| matches!(t, TokenType::LeftParen)) {
            let expr = self.expression()?;
            self.consume(&TokenType::RightParen, "Expect ')' after expression.")?;
            return Ok(Expr::grouping(expr));
        }

        bail!(Lox::error(self.peek(), "Expect expression."))
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
        match cond {
            // We don't care about the contents of the identifier, only the type
            TokenType::Ident(_) if matches!(self.peek().ty, TokenType::Ident(_)) => {
                return Ok(self.advance());
            }
            _ => {
                if self.check(cond) {
                    return Ok(self.advance());
                }
            }
        }

        bail!(Lox::error(self.peek(), err_msg))
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

    fn add_param(&mut self, params: &mut Vec<TokenType>) -> Result<()> {
        if params.len() > 255 {
            bail!(Lox::error(
                self.peek(),
                "Can't have more than 255 parameters"
            ));
        }
        let param = self
            .consume(
                &TokenType::Ident("placeholder".to_string()),
                "Expect parameter name",
            )?
            .expect("unreachable");
        params.push(param.ty);
        Ok(())
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr> {
        let mut args = vec![];
        if !self.check(&TokenType::RightParen) {
            args.push(self.expression()?);
            while self.match_(|t| matches!(t, TokenType::Comma)) {
                if args.len() > 255 {
                    bail!(Lox::error(
                        self.peek(),
                        "Can't have more than 255 function arguments"
                    ))
                }
                args.push(self.expression()?);
            }
        }

        let paren = self
            .consume(&TokenType::RightParen, "Expect ')' after arguments")?
            .unwrap()
            .ty;

        Ok(Expr::function_call(callee, paren, args))
    }

    // TODO: Need to figure out how to incorporate this (see bottom of page 118 in `Crafting Interpreters`)
    #[allow(dead_code)]
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
