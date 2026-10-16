use crate::lox::interpreter::Value;
use anyhow::{Result, bail};
use std::collections::HashMap;

pub struct Environment {
    values: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Result<Value> {
        if let Some(v) = self.values.get(name).cloned() {
            Ok(v)
        } else {
            bail!("Undefined variable '{name}'")
        }
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
