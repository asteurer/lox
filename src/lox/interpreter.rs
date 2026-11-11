use crate::lox::{
    ast::{Expr, Stmt},
    environment::Environment,
    token::TokenType,
};
use anyhow::{Result, bail};
use bumpalo::Bump;

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

#[derive(Clone)]
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

pub struct Interpreter<'a> {
    arena: &'a Bump,
}

impl<'a> Interpreter<'a> {
    pub fn new(arena: &'a Bump) -> Self {
        Interpreter { arena }
    }

    pub fn interpret(&mut self, ast: &[Stmt]) -> Result<()> {
        let env = Environment::new_in(self.arena, None);
        ast.iter().try_for_each(|s| self.execute_statement(s, env))
    }

    fn execute_statement(&self, stmt: &Stmt, env: &'a Environment<'a>) -> Result<()> {
        match stmt {
            Stmt::Expr(expr) => {
                self.evaluate_expr(expr, env)?;
            }
            Stmt::Print(expr) => {
                let val = self.evaluate_expr(expr, env)?;
                println!("{val}");
            }
            Stmt::Var {
                identifier,
                initializer,
            } => {
                let name = match identifier {
                    TokenType::Ident(i) => i.to_owned(),
                    _ => unreachable!(),
                };
                let value = if let Some(expr) = initializer {
                    self.evaluate_expr(expr, env)?
                } else {
                    Value::Nil
                };
                env.define(name, value);
            }
            Stmt::Block(stmts) => {
                let env = Environment::new_in(self.arena, Some(env));
                for stmt in stmts {
                    self.execute_statement(stmt, env)?;
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let env = Environment::new_in(self.arena, Some(env));
                let condition = self.evaluate_expr(&condition, env)?;
                let condition_inner = Interpreter::extract_bool(&condition)?;

                if condition_inner {
                    self.execute_statement(then_branch, env)?;
                }

                if !condition_inner && let Some(stmt) = else_branch {
                    self.execute_statement(stmt, env)?;
                }
            }
            Stmt::While { condition, body } => {
                while Interpreter::extract_bool(&self.evaluate_expr(condition, env)?)? {
                    self.execute_statement(body, env)?;
                }
            }
        }
        Ok(())
    }

    fn evaluate_expr(&self, expr: &Expr, env: &'a Environment<'a>) -> Result<Value> {
        let result = match expr {
            Expr::Literal(tt) => match tt {
                TokenType::Str(s) => Value::String(s.to_owned()),
                TokenType::Num(n) => Value::Number(*n),
                TokenType::False => Value::Bool(false),
                TokenType::True => Value::Bool(true),
                TokenType::Nil => Value::Nil,
                _ => unreachable!(),
            },
            Expr::Variable(tt) => env.get(&tt.lexeme())?,
            Expr::Grouping(expr) => self.evaluate_expr(expr, env)?,
            Expr::Unary { operator, expr } => {
                let resolved = self.evaluate_expr(expr, env)?;
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
                    _ => unreachable!(),
                }
            }
            Expr::Assign { name, value } => {
                let resolved = self.evaluate_expr(value, env)?;
                env.assign(&name.lexeme(), resolved.clone())?;
                resolved
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left_resolved = self.evaluate_expr(&**left, env)?;
                let right_resolved = self.evaluate_expr(&**right, env)?;

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
                    _ => unreachable!(),
                }
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate_expr(left, env)?;
                let left_inner = Interpreter::extract_bool(&left)?;

                if matches!(operator, TokenType::Or) {
                    // Short circuit 'or'
                    if left_inner {
                        return Ok(left);
                    }
                } else {
                    // Short circuit 'and'
                    if !left_inner {
                        return Ok(left);
                    }
                }

                self.evaluate_expr(right, env)?
            }
        };

        Ok(result)
    }

    fn extract_bool(v: &Value) -> Result<bool> {
        match v {
            Value::Bool(b) => Ok(*b),
            _ => bail!("expected a bool type, got {v}"),
        }
    }
}
