use crate::lox::token::TokenType;

pub enum Expr {
    Literal(TokenType),
    Grouping(Box<Expr>),
    Unary {
        operator: TokenType,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: TokenType,
        right: Box<Expr>,
    },
    Assign {
        name: TokenType,
        value: Box<Expr>,
    },
    Variable(TokenType),
}

impl Expr {
    pub fn grouping(expr: Expr) -> Expr {
        Expr::Grouping(Box::new(expr))
    }

    pub fn assign(name: TokenType, value: Expr) -> Expr {
        Expr::Assign {
            name: name,
            value: Box::new(value),
        }
    }

    pub fn unary(operator: TokenType, expr: Expr) -> Expr {
        debug_assert!(matches!(operator, TokenType::Bang | TokenType::Minus));
        Expr::Unary {
            operator,
            expr: Box::new(expr),
        }
    }

    pub fn binary(left: Expr, operator: TokenType, right: Expr) -> Expr {
        debug_assert!(matches!(
            operator,
            TokenType::BangEqual
                | TokenType::EqualEqual
                | TokenType::LessEqual
                | TokenType::Less
                | TokenType::GreaterEqual
                | TokenType::Greater
                | TokenType::Plus
                | TokenType::Minus
                | TokenType::Star
                | TokenType::Slash
        ));

        Expr::Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }

    /// `1 + 2` becomes (+ 1 2)
    /// TODO: This is here for debugging
    pub fn parenthesize(expr: &Expr) -> String {
        let expr_string = match expr {
            Expr::Literal(v) => return v.lexeme(),
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                format!(
                    "{} {} {}",
                    operator.lexeme(),
                    Expr::parenthesize(&**left),
                    Expr::parenthesize(&**right)
                )
            }
            Expr::Grouping(v) => {
                format!("group {}", Expr::parenthesize(&**v),)
            }
            Expr::Unary { operator, expr } => {
                format!("{} {}", operator.lexeme(), Expr::parenthesize(&**expr))
            }
            Expr::Assign { name, value } => {
                format!("assign {} {}", name.lexeme(), Expr::parenthesize(&**value))
            }
            Expr::Variable(name) => format!("var {}", name.lexeme()),
        };

        format!("({expr_string})")
    }
}

pub enum Stmt {
    Expr(Expr),
    Print(Expr),
    Var {
        identifier: TokenType,
        initializer: Option<Expr>,
    },
    Block(Vec<Stmt>),
}

impl Stmt {
    pub fn parenthesize(stmt: &Stmt) -> String {
        let stmt_string = match stmt {
            Stmt::Expr(e) => Expr::parenthesize(e),
            Stmt::Print(e) => format!("print {}", Expr::parenthesize(e)),
            Stmt::Var {
                identifier,
                initializer,
            } => {
                let val = initializer
                    .as_ref()
                    .and_then(|expr| Some(Expr::parenthesize(expr)))
                    .unwrap_or(String::from("nil"));
                format!("var {} {val}", identifier.lexeme())
            }
            Stmt::Block(stmts) => {
                todo!()
            }
        };

        format!("({stmt_string})")
    }
}
