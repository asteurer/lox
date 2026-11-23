use anyhow::{Result, bail};
use std::collections::HashMap;

use crate::lox::{
    Lox,
    ast::{Expr, ExprId, Stmt},
    token::Token,
};

pub struct Resolver;

impl Resolver {
    pub fn resolve(ast: &[Stmt], locals: &mut HashMap<ExprId, usize>) -> Result<()> {
        let mut r = Resolve::new();
        r.begin_scope();
        for stmt in ast {
            Resolver::resolve_stmt(stmt, &mut r, locals)?;
        }
        r.end_scope();

        Ok(())
    }

    fn resolve_stmt(
        stmt: &Stmt,
        resolve: &mut Resolve,
        locals: &mut HashMap<ExprId, usize>,
    ) -> Result<()> {
        match stmt {
            Stmt::Var {
                identifier,
                initializer,
            } => {
                Resolver::declare(resolve, identifier)?;
                if let Some(expr) = initializer {
                    Resolver::resolve_expr(expr, resolve, locals)?;
                }
                Resolver::define(resolve, identifier);
            }
            Stmt::FunctionDef { name, params, body } => {
                Resolver::declare(resolve, name)?;
                Resolver::define(resolve, name);
                Resolver::resolve_function_stmt(params, body, resolve, locals)?;
            }
            Stmt::Expr(expr) => {
                Resolver::resolve_expr(expr, resolve, locals)?;
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                Resolver::resolve_expr(condition, resolve, locals)?;
                Resolver::resolve_stmt(then_branch, resolve, locals)?;
                if let Some(else_) = else_branch {
                    Resolver::resolve_stmt(else_, resolve, locals)?;
                }
            }
            Stmt::Print(expr) => {
                Resolver::resolve_expr(expr, resolve, locals)?;
            }
            Stmt::Return { value, keyword } => {
                if matches!(resolve.fun_type, FunType::None) {
                    bail!(Lox::error(keyword, "Can't return from top-level code"))
                }

                if let Some(v) = value {
                    Resolver::resolve_expr(v, resolve, locals)?;
                }
            }
            Stmt::While { condition, body } => {
                Resolver::resolve_expr(condition, resolve, locals)?;
                Resolver::resolve_stmt(body, resolve, locals)?;
            }
            Stmt::Block(stmts) => {
                for stmt in stmts {
                    Resolver::resolve_stmt(stmt, resolve, locals)?;
                }
            }
        }
        Ok(())
    }

    fn declare(resolve: &mut Resolve, t: &Token) -> Result<()> {
        let last = resolve.scopes.len() - 1;
        if let Some(scope) = resolve.scopes.get_mut(last) {
            if scope.contains_key(&t.ty.lexeme()) {
                bail!(Lox::error(
                    t,
                    "Already a variable with this name in this scope"
                ))
            }

            scope.insert(t.ty.lexeme(), false);
        }
        Ok(())
    }

    fn define(resolve: &mut Resolve, t: &Token) {
        let last = resolve.scopes.len() - 1;
        if let Some(scope) = resolve.scopes.get_mut(last) {
            scope.insert(t.ty.lexeme(), true);
        }
    }

    fn resolve_expr(
        expr: &Expr,
        resolve: &mut Resolve,
        locals: &mut HashMap<ExprId, usize>,
    ) -> Result<()> {
        match expr {
            Expr::Variable { name, id } => {
                if let Some(scope) = resolve.scopes.last()
                    && let Some(defined) = scope.get(&name.ty.lexeme())
                    && !*defined
                {
                    bail!(Lox::error(
                        name,
                        "Can't read local variable in its on initializer."
                    ));
                }

                Resolver::resolve_local(id, resolve, name, locals)?;
            }
            Expr::Assign { name, value, id } => {
                Resolver::resolve_expr(value, resolve, locals)?;
                Resolver::resolve_local(id, resolve, name, locals)?;
            }
            Expr::Binary { left, right, .. } => {
                Resolver::resolve_expr(left, resolve, locals)?;
                Resolver::resolve_expr(right, resolve, locals)?;
            }
            Expr::FunctionCall { callee, args, .. } => {
                Resolver::resolve_expr(callee, resolve, locals)?;
                for arg in args {
                    Resolver::resolve_expr(arg, resolve, locals)?;
                }
            }
            Expr::Grouping(expr) => {
                Resolver::resolve_expr(expr, resolve, locals)?;
            }
            Expr::Literal(_) => {}
            Expr::Logical { left, right, .. } => {
                Resolver::resolve_expr(left, resolve, locals)?;
                Resolver::resolve_expr(right, resolve, locals)?;
            }
            Expr::Unary { expr, .. } => {
                Resolver::resolve_expr(expr, resolve, locals)?;
            }
        }

        Ok(())
    }

    fn resolve_local(
        id: &ExprId,
        resolve: &mut Resolve,
        t: &Token,
        locals: &mut HashMap<ExprId, usize>,
    ) -> Result<()> {
        for (i, scope) in resolve.scopes.iter().rev().enumerate() {
            if scope.contains_key(&t.ty.lexeme()) {
                locals.insert(*id, resolve.scopes.len() - 1 - i);
                break;
            }
        }

        Ok(())
    }

    fn resolve_function_stmt(
        params: &[Token],
        body: &Stmt,
        resolve: &mut Resolve,
        locals: &mut HashMap<ExprId, usize>,
    ) -> Result<()> {
        resolve.fun_type = FunType::Function;
        resolve.begin_scope();

        for param in params {
            Resolver::declare(resolve, param)?;
            Resolver::define(resolve, param);
        }

        Resolver::resolve_stmt(body, resolve, locals)?;
        resolve.end_scope();
        resolve.fun_type = FunType::None;
        Ok(())
    }
}

pub struct Resolve {
    pub scopes: Vec<HashMap<String, bool>>,
    pub fun_type: FunType,
}

impl Resolve {
    pub fn new() -> Self {
        Resolve {
            scopes: vec![],
            fun_type: FunType::None,
        }
    }

    pub fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn end_scope(&mut self) {
        self.scopes.pop();
    }
}

pub enum FunType {
    None,
    Function,
}
