use std::collections::HashMap;

use crate::lox::{
    Lox,
    ast::{Expr, ExprId, Stmt},
    environment::Environment,
    function::{LoxCallable, LoxFunction, native},
    resolver::Resolver,
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
            // TODO: This should be a Lox::error
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
            // TODO: This should be a Lox::error
            (Value::Function(_), _) | (_, Value::Function(_)) => {
                anyhow::bail!("Functions aren't currently comparable.")
            }
        }
    }};
}

macro_rules! parse_control_flow {
    ($stmt:expr) => {
        match $stmt {
            ControlFlow::None => {}
            return_val => return Ok(return_val),
        }
    };
}

#[derive(Clone, Debug)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
    Function(Callable),
}

// TODO: Need to store a line number for callables
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

pub enum ControlFlow {
    Return(Value),
    None,
}

pub struct Interpreter;

impl Interpreter {
    pub fn interpret(arena: &Bump, enclosing: Option<&Environment>, ast: &[Stmt]) -> Result<()> {
        let mut locals: HashMap<ExprId, usize> = HashMap::new();
        Resolver::resolve(ast, &mut locals)?;

        let env = Environment::new_in(arena, enclosing);
        Interpreter::add_native_functions(env)?;

        for stmt in ast {
            println!("STATEMENT: {}", Stmt::parenthesize(stmt));
            Interpreter::execute_statement(arena, env, stmt, &mut locals)?;
        }

        Ok(())
    }

    pub fn execute_statement(
        arena: &Bump,
        env: &Environment,
        stmt: &Stmt,
        locals: &mut HashMap<ExprId, usize>,
    ) -> Result<ControlFlow> {
        match stmt {
            Stmt::Expr(expr) => {
                Interpreter::evaluate_expr(arena, env, expr, locals)?;
            }
            Stmt::Print(expr) => {
                let val = Interpreter::evaluate_expr(arena, env, expr, locals)?;
                println!("{val}");
            }
            Stmt::Var {
                identifier,
                initializer,
            } => {
                let name = match &identifier.ty {
                    TokenType::Ident(i) => i.to_owned(),
                    _ => unreachable!(),
                };
                let value = if let Some(expr) = initializer {
                    Interpreter::evaluate_expr(arena, env, expr, locals)?
                } else {
                    Value::Nil
                };
                env.define(name, value);
            }
            Stmt::Block(stmts) => {
                let env = Environment::new_in(arena, Some(env));
                for stmt in stmts {
                    parse_control_flow!(Interpreter::execute_statement(arena, env, stmt, locals)?)
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let env = Environment::new_in(arena, Some(env));
                let condition = Interpreter::evaluate_expr(arena, env, &condition, locals)?;
                let condition_inner = Interpreter::extract_bool(&condition)?;

                if condition_inner {
                    parse_control_flow!(Interpreter::execute_statement(
                        arena,
                        env,
                        then_branch,
                        locals
                    )?)
                }

                if !condition_inner && let Some(stmt) = else_branch {
                    parse_control_flow!(Interpreter::execute_statement(arena, env, stmt, locals)?)
                }
            }
            Stmt::While { condition, body } => {
                while Interpreter::extract_bool(&Interpreter::evaluate_expr(
                    arena, env, condition, locals,
                )?)? {
                    parse_control_flow!(Interpreter::execute_statement(arena, env, body, locals)?)
                }
            }
            Stmt::FunctionDef { name, .. } => {
                env.define(
                    name.ty.lexeme(),
                    Value::Function(Callable::Lox(LoxFunction::new(stmt))),
                );
            }
            Stmt::Return { value, .. } => {
                let mut ret_val = ControlFlow::None;
                if let Some(v) = value {
                    ret_val =
                        ControlFlow::Return(Interpreter::evaluate_expr(arena, env, v, locals)?);
                }

                return Ok(ret_val);
            }
        }

        Ok(ControlFlow::None)
    }

    fn evaluate_expr(
        arena: &Bump,
        env: &Environment,
        expr: &Expr,
        locals: &mut HashMap<ExprId, usize>,
    ) -> Result<Value> {
        let result = match expr {
            Expr::Literal(t) => match &t.ty {
                TokenType::Str(s) => Value::String(s.to_owned()),
                TokenType::Num(n) => Value::Number(*n),
                TokenType::False => Value::Bool(false),
                TokenType::True => Value::Bool(true),
                TokenType::Nil => Value::Nil,
                _ => unreachable!(),
            },
            Expr::Variable { name, id } => {
                if let Some(distance) = locals.get(id) {
                    env.get_at(distance, name.ty.lexeme())?
                } else {
                    env.get(&name.ty.lexeme())?
                }
            }
            Expr::Grouping(expr) => Interpreter::evaluate_expr(arena, env, expr, locals)?,
            Expr::Unary { operator, expr } => {
                let resolved = Interpreter::evaluate_expr(arena, env, expr, locals)?;
                match operator.ty {
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
                            bail!(Lox::error(
                                operator,
                                "The minus (`-`) unary operator can only be used on numbers"
                            ));
                        }
                    }
                    _ => unreachable!(),
                }
            }
            Expr::Assign { name, value, id } => {
                let val = Interpreter::evaluate_expr(arena, env, value, locals)?;
                if let Some(distance) = locals.get(id) {
                    env.assign_at(distance, &name.ty.lexeme(), val.clone())?;
                } else {
                    env.assign(&name.ty.lexeme(), val.clone())?;
                }
                val
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left_resolved = Interpreter::evaluate_expr(arena, env, &**left, locals)?;
                let right_resolved = Interpreter::evaluate_expr(arena, env, &**right, locals)?;

                match operator.ty {
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
                        _ => bail!(Lox::error(
                            operator,
                            "The plus (`+`) binary operator can only be used with numbers or strings"
                        )),
                    },
                    _ => unreachable!(),
                }
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = Interpreter::evaluate_expr(arena, env, left, locals)?;
                let left_inner = Interpreter::extract_bool(&left)?;

                if matches!(operator.ty, TokenType::Or) {
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

                Interpreter::evaluate_expr(arena, env, right, locals)?
            }
            Expr::FunctionCall {
                callee,
                args,
                paren,
            } => {
                let mut resolved_args = vec![];
                for arg in args {
                    resolved_args.push(Interpreter::evaluate_expr(arena, env, arg, locals)?);
                }

                match Interpreter::evaluate_expr(arena, env, callee, locals)? {
                    Value::Function(callable) => match callable {
                        Callable::Lox(f) => {
                            if args.len() as u8 != f.arity() {
                                bail!(Lox::error(
                                    paren,
                                    &format!(
                                        "Expected {} arguments but got {}",
                                        f.arity(),
                                        args.len()
                                    )
                                ))
                            }
                            let res = f.call(arena, Some(env), &resolved_args, locals)?;
                            return Ok(res);
                        }
                        Callable::Native { name, arity, fun } => {
                            if args.len() as u8 != arity {
                                // TODO: This should be a Lox::error
                                bail!(
                                    "Native function '{name}' expects {} arguments but got {}",
                                    arity,
                                    args.len()
                                )
                            }

                            return Ok(fun(&resolved_args)?);
                        }
                    },
                    _ => bail!(Lox::error(paren, "Can only call functions and classes")),
                }
            }
        };

        Ok(result)
    }

    fn extract_bool(v: &Value) -> Result<bool> {
        match v {
            Value::Bool(b) => Ok(*b),
            // TODO: This should be a Lox::error
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
