use std::collections::{hash_map::Entry, HashMap};

use super::{error::ResolverError, expr::Expr, scanner::Token, stmt::Stmt, INTERPRETER};

pub type ResolverResult = Result<(), ResolverError>;

#[derive(Default, Copy, Clone)]
enum FunctionType {
    #[default]
    None,
    Function,
}

#[derive(Default)]
pub struct Resolver {
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
}

impl Resolver {
    pub fn resolve_stmts(&mut self, stmts: &[Stmt]) -> ResolverResult {
        for stmt in stmts {
            self.resolve_stmt(stmt)?;
        }
        Ok(())
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) -> ResolverResult {
        match stmt {
            Stmt::Block(stmts) => {
                self.begin_scope();
                self.resolve_stmts(stmts)?;
                self.end_scope();
                Ok(())
            }
            Stmt::Expression(expr) => self.resolve_expr(expr),
            Stmt::Function { name, params, body } => {
                self.declare(name)?;
                self.define(name);
                self.resolve_function(params, body, FunctionType::Function)
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expr(condition)?;
                self.resolve_stmt(then_branch)?;
                if let Some(else_branch) = else_branch {
                    self.resolve_stmt(else_branch)
                } else {
                    Ok(())
                }
            }
            Stmt::Print(expr) => self.resolve_expr(expr),
            Stmt::Return { keyword, value } => {
                if let FunctionType::None = self.current_function {
                    return Err(ResolverError::new(
                        keyword.to_owned(),
                        "Can't return from top-level code.".to_string(),
                    ));
                }
                if let Some(expr) = value {
                    self.resolve_expr(expr)
                } else {
                    Ok(())
                }
            }
            Stmt::Var { name, initializer } => {
                self.declare(name)?;
                if let Some(initializer) = initializer {
                    self.resolve_expr(initializer)?;
                }
                self.define(name);
                Ok(())
            }
            Stmt::While { condition, body } => {
                self.resolve_expr(condition)?;
                self.resolve_stmt(body)
            }
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) -> ResolverResult {
        match expr {
            Expr::Assign { id, name, value } => {
                self.resolve_expr(value)?;
                self.resolve_local(id, name)
            }
            Expr::Binary {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)
            }
            Expr::Call {
                callee,
                paren: _,
                arguments,
            } => {
                self.resolve_expr(callee)?;
                for arg in arguments {
                    self.resolve_expr(arg)?;
                }
                Ok(())
            }
            Expr::Grouping(expr) => self.resolve_expr(expr),
            Expr::Literal(_) => Ok(()),
            Expr::Logical {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)
            }
            Expr::Unary { operator: _, right } => self.resolve_expr(right),
            Expr::Variable { id, name } => {
                if !self.scopes.is_empty()
                    && self.scopes.last().unwrap().get(&name.lexeme) == Some(&false)
                {
                    Err(ResolverError::new(
                        name.to_owned(),
                        "Can't read local variable in its own initializer.".to_string(),
                    ))
                } else {
                    self.resolve_local(id, name)
                }
            }
        }
    }

    fn resolve_function(
        &mut self,
        params: &[Token],
        body: &[Stmt],
        func_type: FunctionType,
    ) -> ResolverResult {
        let enclosing_function = self.current_function;
        self.current_function = func_type;
        self.begin_scope();
        for param in params {
            self.declare(param)?;
            self.define(param);
        }
        self.resolve_stmts(body)?;
        self.end_scope();
        self.current_function = enclosing_function;
        Ok(())
    }

    fn resolve_local(&mut self, id: &usize, name: &Token) -> ResolverResult {
        for (depth, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme) {
                INTERPRETER
                    .lock()
                    .expect("Unable to lock INTERPRETER.")
                    .resolve(id, depth)?;
            }
        }
        Ok(())
    }

    fn declare(&mut self, name: &Token) -> ResolverResult {
        if self.scopes.is_empty() {
            Ok(())
        } else {
            match self.scopes.last_mut().unwrap().entry(name.lexeme.clone()) {
                Entry::Occupied(_) => Err(ResolverError::new(
                    name.to_owned(),
                    "Already a variable with this name in this scope.".to_string(),
                )),
                Entry::Vacant(entry) => {
                    entry.insert(false);
                    Ok(())
                }
            }
        }
    }

    fn define(&mut self, name: &Token) {
        if !self.scopes.is_empty() {
            self.scopes
                .last_mut()
                .unwrap()
                .insert(name.lexeme.clone(), true);
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop().expect("Attempted to pop empty 'scopes'.");
    }
}
