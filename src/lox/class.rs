use crate::lox::{
    Lox,
    environment::Environment,
    function::{LoxCallable, LoxFunction},
    interpreter::{Callable, Value},
    token::Token,
};
use anyhow::{Result, bail};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Class {
    pub name: String,
    pub methods: HashMap<String, LoxFunction>,
}

impl Class {
    pub fn new(name: String, methods: HashMap<String, LoxFunction>) -> Self {
        Class { name, methods }
    }
}

impl std::fmt::Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl LoxCallable for Class {
    fn arity(&self) -> u8 {
        0
    }

    fn call(
        &self,
        _arena: &bumpalo::Bump,
        _env: Option<&Environment>,
        _args: &[Value],
        _locals: &mut std::collections::HashMap<super::ast::ExprId, usize>,
    ) -> anyhow::Result<Value> {
        Ok(Value::ClassInstance(Instance {
            class: self.clone(),
            fields: HashMap::new(),
        }))
    }
}

#[derive(Clone, Debug)]
pub struct Instance {
    pub class: Class,
    pub fields: HashMap<String, Value>,
}

impl Instance {
    pub fn get(&self, name: &Token) -> Result<Value> {
        if let Some(v) = self.fields.get(&name.ty.lexeme()) {
            Ok(v.clone())
        } else if let Some(f) = self.class.methods.get(&name.ty.lexeme()).cloned() {
            Ok(Value::Function(Callable::Lox(f)))
        } else {
            bail!(Lox::error(
                name,
                &format!("Undefined property {}", name.ty.lexeme())
            ))
        }
    }

    pub fn set(&mut self, name: &Token, value: Value) {
        self.fields
            .insert(name.ty.lexeme(), value)
            .expect("failed to insert value into fields map");
    }
}
