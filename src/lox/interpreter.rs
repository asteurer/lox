use crate::lox::{
    ast::{Expr, Stmt},
    environment::Environment,
    function::{LoxCallable, LoxFunction, native},
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
            (Value::Function(_), _) | (_, Value::Function(_)) => {
                anyhow::bail!("Functions aren't currently comparable.")
            }
        }
    }};
}

#[derive(Clone, Debug)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
    Function(Callable),
}

#[derive(Clone, Debug)]
pub enum Callable {
    Lox(LoxFunction),
    Native {
        name: &'static str,
        arity: u8,
        fun: fn(&[Value]) -> Result<Value>,
    },
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(b) => write!(f, "{b}"),
            Value::Number(n) => write!(f, "{n}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Nil => write!(f, "nil"),
            Value::Function(callable) => match callable {
                Callable::Lox(fun) => write!(f, "{fun}",),
                Callable::Native { name, .. } => write!(f, "<native fun {name}>"),
            },
        }
    }
}

pub struct Interpreter;

impl Interpreter {
    pub fn interpret(arena: &Bump, enclosing: Option<&Environment>, ast: &[Stmt]) -> Result<()> {
        let env = Environment::new_in(arena, enclosing);
        Interpreter::add_native_functions(env)?;

        ast.iter().try_for_each(|s| {
            // println!("{}", Stmt::parenthesize(s));
            Interpreter::execute_statement(arena, env, s)
        })
    }

    fn execute_statement(arena: &Bump, env: &Environment, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Expr(expr) => {
                Interpreter::evaluate_expr(arena, env, expr)?;
            }
            Stmt::Print(expr) => {
                let val = Interpreter::evaluate_expr(arena, env, expr)?;
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
                    Interpreter::evaluate_expr(arena, env, expr)?
                } else {
                    Value::Nil
                };
                env.define(name, value);
            }
            Stmt::Block(stmts) => {
                let env = Environment::new_in(arena, Some(env));
                for stmt in stmts {
                    Interpreter::execute_statement(arena, env, stmt)?;
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let env = Environment::new_in(arena, Some(env));
                let condition = Interpreter::evaluate_expr(arena, env, &condition)?;
                let condition_inner = Interpreter::extract_bool(&condition)?;

                if condition_inner {
                    Interpreter::execute_statement(arena, env, then_branch)?;
                }

                if !condition_inner && let Some(stmt) = else_branch {
                    Interpreter::execute_statement(arena, env, stmt)?;
                }
            }
            Stmt::While { condition, body } => {
                while Interpreter::extract_bool(&Interpreter::evaluate_expr(
                    arena, env, condition,
                )?)? {
                    Interpreter::execute_statement(arena, env, body)?;
                }
            }
            Stmt::FunctionDef { name, .. } => {
                env.define(
                    name.lexeme(),
                    Value::Function(Callable::Lox(LoxFunction::new(stmt))),
                );
            }
        }
        Ok(())
    }

    fn evaluate_expr(arena: &Bump, env: &Environment, expr: &Expr) -> Result<Value> {
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
            Expr::Grouping(expr) => Interpreter::evaluate_expr(arena, env, expr)?,
            Expr::Unary { operator, expr } => {
                let resolved = Interpreter::evaluate_expr(arena, env, expr)?;
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
                let resolved = Interpreter::evaluate_expr(arena, env, value)?;
                env.assign(&name.lexeme(), resolved.clone())?;
                resolved
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left_resolved = Interpreter::evaluate_expr(arena, env, &**left)?;
                let right_resolved = Interpreter::evaluate_expr(arena, env, &**right)?;

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
                let left = Interpreter::evaluate_expr(arena, env, left)?;
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

                Interpreter::evaluate_expr(arena, env, right)?
            }
            Expr::FunctionCall { callee, args, .. } => {
                let mut resolved_args = vec![];
                for arg in args {
                    resolved_args.push(Interpreter::evaluate_expr(arena, env, arg)?);
                }

                match Interpreter::evaluate_expr(arena, env, callee)? {
                    Value::Function(callable) => match callable {
                        Callable::Lox(f) => {
                            if args.len() as u8 != f.arity() {
                                bail!("Expected {} arguments but got {}", f.arity(), args.len())
                            }
                            let res = f.call(arena, Some(env), &resolved_args)?;
                            res
                        }
                        Callable::Native { name, arity, fun } => {
                            if args.len() as u8 != arity {
                                bail!(
                                    "Native function '{name}' expects {} arguments but got {}",
                                    arity,
                                    args.len()
                                )
                            }

                            fun(&resolved_args)?
                        }
                    },
                    _ => bail!("Can only call functions and classes"),
                }
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

    fn add_native_functions(env: &Environment) -> Result<()> {
        env.define(
            "clock".to_string(),
            Value::Function(Callable::Native {
                name: "clock",
                arity: 0,
                fun: native::clock,
            }),
        );

        env.define(
            "sleep".to_string(),
            Value::Function(Callable::Native {
                name: "sleep",
                arity: 1,
                fun: native::sleep,
            }),
        );
        Ok(())
    }
}
