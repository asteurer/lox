use crate::lox::interpreter::Value;
use anyhow::{Result, bail};
use bumpalo::Bump;
use std::cell::RefCell;
use std::collections::HashMap;

pub struct Environment<'a> {
    values: RefCell<HashMap<String, Value>>,
    enclosing: Option<&'a Environment<'a>>,
}

impl<'a> Environment<'a> {
    pub fn new_in(arena: &'a Bump, enclosing: Option<&'a Environment<'a>>) -> &'a Environment<'a> {
        arena.alloc(Environment {
            values: RefCell::new(HashMap::new()),
            enclosing,
        })
    }

    pub fn define(&self, name: String, value: Value) {
        self.values.borrow_mut().insert(name, value);
    }

    pub fn get(&self, name: &str) -> Result<Value> {
        if let Some(v) = self.values.borrow().get(name).cloned() {
            // If the value is contained in the immediate scope...
            Ok(v)
        } else if let Some(env) = self.enclosing.as_deref() {
            // If the value is contained in a parent/ancestor's scope...
            env.get(name)
        } else {
            bail!("Undefined variable '{name}'")
        }
    }

    pub fn assign(&self, name: &str, value: Value) -> Result<()> {
        match (self.values.borrow_mut().get_mut(name), self.enclosing) {
            (Some(v), _) => *v = value,
            (None, Some(enclosing)) => enclosing.assign(name, value)?,
            _ => bail!("Undefined variable '{name}'"),
        }
        Ok(())
    }

    /// TODO: This is here for debugging
    #[allow(dead_code)]
    pub fn dump(&self) -> String {
        self.values
            .borrow()
            .iter()
            .map(|(k, v)| format!("{k}: {v}"))
            .collect::<Vec<String>>()
            .join(", ")
    }
}

impl<'a> Drop for Environment<'a> {
    fn drop(&mut self) {
        self.values.take();
    }
}
