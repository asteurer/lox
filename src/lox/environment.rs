use crate::lox::interpreter::Value;
use anyhow::{Result, bail};
use std::collections::HashMap;

pub struct Environment<'a> {
    values: HashMap<String, Value>,
    /// Pointer to parent enclosure
    enclosing: Option<Box<&'a mut Environment<'a>>>,
}

impl<'a> Environment<'a> {
    pub fn new(enclosing: Option<&'a mut Environment<'a>>) -> Self {
        Environment {
            values: HashMap::new(),
            enclosing: enclosing.map(|e| Box::new(e)),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Result<Value> {
        if let Some(v) = self.values.get(name).cloned() {
            // If the value is contained in the immediate scope...
            Ok(v)
        } else if let Some(env) = self.enclosing.as_ref() {
            // If the value is contained in a parent/ancestor's scope...
            env.get(name)
        } else {
            bail!("Undefined variable '{name}'")
        }
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Result<()> {
        match (self.values.get_mut(name), self.enclosing.as_mut()) {
            (Some(v), _) => *v = value,
            (None, Some(env)) => env.assign(name, value)?,
            _ => bail!("Undefined variable '{name}'"),
        }
        Ok(())
    }

    /// TODO: This is here for debugging
    #[allow(dead_code)]
    pub fn dump(&self) -> String {
        self.values
            .iter()
            .map(|(k, v)| format!("{k}: {v}"))
            .collect::<Vec<String>>()
            .join(", ")
    }
}
