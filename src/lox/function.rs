use std::fmt::Display;

use crate::lox::{
    ast::Stmt,
    environment::Environment,
    interpreter::{Interpreter, Value},
};
use anyhow::{Result, bail};
use bumpalo::Bump;

pub trait LoxCallable {
    fn call(&self, arena: &Bump, env: Option<&Environment>, args: &[Value]) -> Result<Value>;
    fn arity(&self) -> u8;
}

#[derive(Clone, Debug)]
pub struct LoxFunction {
    fun_decl: Stmt,
}

impl LoxFunction {
    pub fn new(fun: &Stmt) -> Self {
        LoxFunction {
            fun_decl: fun.clone(),
        }
    }
}

impl LoxCallable for LoxFunction {
    fn arity(&self) -> u8 {
        match &self.fun_decl {
            Stmt::FunctionDef { params, .. } => params.len() as u8,
            _ => unreachable!(),
        }
    }

    fn call(&self, arena: &Bump, env: Option<&Environment>, args: &[Value]) -> Result<Value> {
        let env = Environment::new_in(arena, env);
        if let Stmt::FunctionDef { params, body, .. } = &self.fun_decl {
            for i in 0..params.len() {
                env.define(
                    params.get(i).unwrap().lexeme(),
                    args.get(i).unwrap().clone(),
                );
            }

            Interpreter::interpret(arena, Some(env), std::slice::from_ref(body))?;
            Ok(Value::Nil)
        } else {
            bail!("Expected LoxFunction.func_decl to be type Stmt::Function");
        }
    }
}

impl Display for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match &self.fun_decl {
            Stmt::FunctionDef { name, .. } => name.lexeme(),
            _ => unreachable!(),
        };

        write!(f, "<fn {name}>",)?;
        Ok(())
    }
}

pub mod native {
    use std::time::Duration;

    use crate::lox::interpreter::Value;
    use anyhow::{Result, bail};

    pub fn clock(_args: &[Value]) -> Result<Value> {
        let millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is before the Unix epoch")
            .as_millis(); // u128

        Ok(Value::Number(millis as f64))
    }

    pub fn sleep(args: &[Value]) -> Result<Value> {
        let duration = match &args[0] {
            Value::Number(n) => n,
            other => bail!("Expected number, got {other}"),
        };
        std::thread::sleep(Duration::from_millis(*duration as u64));
        Ok(Value::Nil)
    }
}
