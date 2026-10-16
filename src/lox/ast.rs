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
}

impl Expr {
    pub fn grouping(expr: Expr) -> Expr {
        Expr::Grouping(Box::new(expr))
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
    #[allow(dead_code)]
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
}
