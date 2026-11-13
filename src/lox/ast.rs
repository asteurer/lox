use crate::lox::token::TokenType;

#[derive(Clone, Debug)]
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
    Logical {
        left: Box<Expr>,
        operator: TokenType,
        right: Box<Expr>,
    },
    FunctionCall {
        callee: Box<Expr>,
        paren: TokenType,
        args: Vec<Expr>,
    },
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

    pub fn logical(left: Expr, operator: TokenType, right: Expr) -> Expr {
        debug_assert!(matches!(operator, TokenType::And | TokenType::Or));
        Expr::Logical {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }

    pub fn function_call(callee: Expr, paren: TokenType, args: Vec<Expr>) -> Expr {
        Expr::FunctionCall {
            callee: Box::new(callee),
            paren,
            args,
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
            Expr::Logical {
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
            Expr::FunctionCall { callee, args, .. } => {
                format!(
                    "call fun {}({})",
                    Expr::parenthesize(callee),
                    args.iter()
                        .map(|a| Expr::parenthesize(a))
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
        };

        format!("({expr_string})")
    }
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Expr(Expr),
    Print(Expr),
    Var {
        identifier: TokenType,
        initializer: Option<Expr>,
    },
    Block(Vec<Stmt>),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    FunctionDef {
        name: TokenType,
        params: Vec<TokenType>,
        body: Box<Stmt>,
    },
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
                let mut statements = String::new();
                for stmt in stmts {
                    statements = format!("{statements}\n{}", Stmt::parenthesize(stmt))
                }
                statements
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let branches = if let Some(else_) = else_branch {
                    format!(
                        r#"then {}
else {}"#,
                        Stmt::parenthesize(then_branch),
                        Stmt::parenthesize(else_)
                    )
                } else {
                    format!("then {}", Stmt::parenthesize(then_branch))
                };
                format!(
                    r#"if (
  condition {}
  {}
)"#,
                    Expr::parenthesize(condition),
                    branches
                )
            }
            Stmt::While { condition, body } => {
                format!(
                    r#"while (
  condition {}
  body {}
)"#,
                    Expr::parenthesize(condition),
                    Stmt::parenthesize(body)
                )
            }
            Stmt::FunctionDef { name, params, body } => {
                let params = params
                    .iter()
                    .map(|p| format!("{}", p.lexeme()))
                    .collect::<Vec<String>>()
                    .join(", ");
                format!(
                    r#"function {}(
  params ({})
  body {}
)"#,
                    name.lexeme(),
                    params,
                    Stmt::parenthesize(body)
                )
            }
        };

        format!("({stmt_string})")
    }
}
