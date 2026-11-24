use crate::lox::token::{Token, TokenType};

pub type ExprId = usize;

#[derive(Clone, Debug)]
pub enum Expr {
    Literal(Token),
    Grouping(Box<Expr>),
    Unary {
        operator: Token,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Assign {
        id: ExprId,
        name: Token,
        value: Box<Expr>,
    },
    Variable {
        id: ExprId,
        name: Token,
    },
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    FunctionCall {
        callee: Box<Expr>,
        paren: Token,
        args: Vec<Expr>,
    },
    Get {
        object: Box<Expr>,
        name: Token,
    },
    Set {
        value: Box<Expr>,
        object: Box<Expr>,
        name: Token,
    },
}

impl Expr {
    pub fn grouping(expr: Expr) -> Expr {
        Expr::Grouping(Box::new(expr))
    }

    pub fn assign(name: Token, value: Expr, id: ExprId) -> Expr {
        Expr::Assign {
            id,
            name,
            value: Box::new(value),
        }
    }

    pub fn unary(operator: Token, expr: Expr) -> Expr {
        debug_assert!(matches!(operator.ty, TokenType::Bang | TokenType::Minus));
        Expr::Unary {
            operator,
            expr: Box::new(expr),
        }
    }

    pub fn binary(left: Expr, operator: Token, right: Expr) -> Expr {
        debug_assert!(matches!(
            operator.ty,
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

    pub fn logical(left: Expr, operator: Token, right: Expr) -> Expr {
        debug_assert!(matches!(operator.ty, TokenType::And | TokenType::Or));
        Expr::Logical {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }

    pub fn get(object: Expr, name: Token) -> Expr {
        Expr::Get {
            object: Box::new(object),
            name,
        }
    }

    pub fn set(value: Expr, object: Expr, name: Token) -> Expr {
        Expr::Set {
            value: Box::new(value),
            object: Box::new(object),
            name,
        }
    }

    pub fn function_call(callee: Expr, paren: Token, args: Vec<Expr>) -> Expr {
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
            Expr::Literal(v) => return v.ty.lexeme(),
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                format!(
                    "{} {} {}",
                    operator.ty.lexeme(),
                    Expr::parenthesize(&**left),
                    Expr::parenthesize(&**right)
                )
            }
            Expr::Grouping(v) => {
                format!("group {}", Expr::parenthesize(&**v),)
            }
            Expr::Unary { operator, expr } => {
                format!("{} {}", operator.ty.lexeme(), Expr::parenthesize(&**expr))
            }
            Expr::Assign { name, value, .. } => {
                format!(
                    "assign {} {}",
                    name.ty.lexeme(),
                    Expr::parenthesize(&**value)
                )
            }
            Expr::Variable { name, .. } => format!("var {}", name.ty.lexeme()),
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                format!(
                    "{} {} {}",
                    operator.ty.lexeme(),
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
            Expr::Get { object, name } => {
                format!("TODO: Get Expressions")
            }
            Expr::Set {
                value,
                object,
                name,
            } => {
                format!("TODO: Set Expressions")
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
        identifier: Token,
        initializer: Option<Expr>,
    },
    Return {
        keyword: Token,
        value: Option<Expr>,
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
        name: Token,
        params: Vec<Token>,
        body: Box<Stmt>,
    },
    Class {
        name: Token,
        methods: Vec<Stmt>,
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
                format!("var {} {val}", identifier.ty.lexeme())
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
                    .map(|p| format!("{}", p.ty.lexeme()))
                    .collect::<Vec<String>>()
                    .join(", ");
                format!(
                    r#"function {}(
  params ({})
  body {}
)"#,
                    name.ty.lexeme(),
                    params,
                    Stmt::parenthesize(body)
                )
            }
            Stmt::Return { value, .. } => {
                if let Some(v) = value {
                    return format!("return {}", Expr::parenthesize(v));
                }

                format!("return")
            }
            Stmt::Class { name, methods } => {
                format!("TODO: Class")
            }
        };

        format!("({stmt_string})")
    }
}
