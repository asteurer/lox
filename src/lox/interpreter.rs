use crate::lox::{ast::Expr, token::TokenType};
use anyhow::{Result, bail};

/// Evaluates numeric binary operations between two number types that evaluate to a number type or boolean type
macro_rules! binary_number {
    ($ctor:ident, $op:tt, $lhs:expr, $rhs:expr) => {{
        match ($lhs, $rhs) {
            (Value::Number(left), Value::Number(right)) => {
                return Ok(Value::$ctor(left $op right));
            }
            _ => bail!("The `{}` operator can only be used on numbers", stringify!($op))
        }
    }};
}

/// Evaluates comparison binary operations between two types that evaluate to a boolean type
macro_rules! binary_equality {
    ($op:tt, $lhs:expr, $rhs:expr) => {{
        match ($lhs, $rhs) {
            (Value::Nil, Value::Nil) => {
                return Ok(Value::Bool(false $op false));
            }
            (Value::Nil, Value::Bool(b)) => {
                return Ok(Value::Bool(false $op b));
            }
            (Value::Bool(b), Value::Nil) => {
                return Ok(Value::Bool(b $op false));
            }
            (Value::Number(left), Value::Number(right)) => {
                return Ok(Value::Bool(left $op right));
            }
            (Value::Bool(left), Value::Bool(right)) => {
                return Ok(Value::Bool(left $op right));
            }
            (Value::String(left), Value::String(right)) => {
                return Ok(Value::Bool(left $op right));
            }
            (Value::Nil, _)
            | (_, Value::Nil)
            | (Value::Number(_), Value::String(_) | Value::Bool(_))
            | (Value::String(_), Value::Number(_) | Value::Bool(_))
            | (Value::Bool(_), Value::String(_) | Value::Number(_)) => {
                return Ok(Value::Bool(true $op false));
            }
        }
    }};
}

pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(b) => write!(f, "{b}"),
            Value::Number(n) => write!(f, "{n}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Nil => write!(f, "nil"),
        }
    }
}

pub fn evaluate(expr: &Expr) -> Result<Value> {
    let result = match expr {
        Expr::Literal(tt) => match tt {
            TokenType::Str(s) => Value::String(s.to_owned()),
            TokenType::Num(n) => Value::Number(*n),
            TokenType::False => Value::Bool(false),
            TokenType::True => Value::Bool(true),
            TokenType::Nil => Value::Nil,
            other => panic!(
                "This is incorrectly being interpreted as a literal expression: '{}'",
                other.lexeme()
            ),
        },
        Expr::Grouping(expr) => evaluate(expr)?,
        Expr::Unary { operator, expr } => {
            let resolved = evaluate(expr)?;
            match operator {
                TokenType::Bang => {
                    let val = match resolved {
                        Value::Nil => false,
                        Value::Bool(b) => b,
                        _ => true,
                    };
                    return Ok(Value::Bool(!val));
                }
                TokenType::Minus => {
                    if let Value::Number(n) = resolved {
                        Value::Number(-n)
                    } else {
                        bail!("The minus (`-`) unary operator can only be used on numbers");
                    }
                }
                other => panic!(
                    "This is incorrectly being interpreted as a unary operator: '{}'",
                    other.lexeme()
                ),
            }
        }
        Expr::Binary {
            left,
            operator,
            right,
        } => {
            let left_resolved = evaluate(&**left)?;
            let right_resolved = evaluate(&**right)?;

            match operator {
                TokenType::Minus => {
                    binary_number!(Number, -, left_resolved, right_resolved)
                }
                TokenType::Slash => {
                    binary_number!(Number, /, left_resolved, right_resolved)
                }
                TokenType::Star => {
                    binary_number!(Number, *, left_resolved, right_resolved)
                }
                TokenType::EqualEqual => {
                    binary_equality!(==, left_resolved, right_resolved)
                }
                TokenType::BangEqual => {
                    binary_equality!(!=, left_resolved, right_resolved)
                }
                TokenType::Greater => {
                    binary_number!(Bool, >, left_resolved, right_resolved)
                }
                TokenType::GreaterEqual => {
                    binary_number!(Bool, >=, left_resolved, right_resolved)
                }
                TokenType::Less => {
                    binary_number!(Bool, <, left_resolved, right_resolved)
                }
                TokenType::LessEqual => {
                    binary_number!(Bool, <=, left_resolved, right_resolved)
                }
                TokenType::Plus => match (left_resolved, right_resolved) {
                    (Value::String(lhs), Value::String(rhs)) => {
                        return Ok(Value::String(format!("{lhs}{rhs}")));
                    }
                    (Value::Number(lhs), Value::Number(rhs)) => {
                        return Ok(Value::Number(lhs + rhs));
                    }
                    _ => bail!(
                        "The plus (`+`) binary operator can only be used with numbers or strings"
                    ),
                },
                _ => todo!(),
            }
        }
    };

    Ok(result)
}
