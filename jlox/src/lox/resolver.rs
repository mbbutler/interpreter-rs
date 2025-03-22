use std::collections::{hash_map::Entry, HashMap};

use super::{
    error::ResolverError,
    expr::Expr,
    interpreter::Interpreter,
    scanner::Token,
    stmt::{Function, Stmt},
};

pub type ResolverResult = Result<(), ResolverError>;

#[derive(Default, Copy, Clone)]
enum FunctionType {
    #[default]
    None,
    Function,
    Initializer,
    Method,
}

#[derive(Default, Copy, Clone)]
enum ClassType {
    #[default]
    None,
    Class,
    Subclass,
}

pub struct Resolver<'a> {
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
    interpreter: &'a mut Interpreter,
    current_class: ClassType,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            scopes: Vec::new(),
            current_function: FunctionType::None,
            interpreter,
            current_class: ClassType::None,
        }
    }

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
            Stmt::Class {
                name,
                methods,
                superclass,
            } => {
                let enclosing_class = self.current_class;
                self.current_class = ClassType::Class;
                self.declare(name)?;
                self.define(name);
                if let Some(superclass) = superclass {
                    if let Expr::Variable {
                        id: _,
                        name: sc_name,
                    } = superclass
                    {
                        if name.lexeme == sc_name.lexeme {
                            return Err(ResolverError::new(
                                sc_name.to_owned(),
                                "A class can't inherit from itself.".to_string(),
                            ));
                        }
                    }
                    self.current_class = ClassType::Subclass;
                    self.resolve_expr(superclass)?;
                }

                if superclass.is_some() {
                    self.begin_scope();
                    self.scopes
                        .last_mut()
                        .unwrap()
                        .insert("super".to_string(), true);
                }

                self.begin_scope();
                self.scopes
                    .last_mut()
                    .expect("Scopes is empty")
                    .insert("this".to_string(), true);
                for method in methods {
                    let declaration = if &method.name.lexeme == "init" {
                        FunctionType::Initializer
                    } else {
                        FunctionType::Method
                    };
                    self.resolve_function(method, declaration)?;
                }
                self.end_scope();

                if superclass.is_some() {
                    self.end_scope();
                }

                self.current_class = enclosing_class;
                Ok(())
            }
            Stmt::Expression(expr) => self.resolve_expr(expr),
            Stmt::Function(func) => {
                self.declare(&func.name)?;
                self.define(&func.name);
                self.resolve_function(func, FunctionType::Function)
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
                    if let FunctionType::Initializer = self.current_function {
                        return Err(ResolverError::new(
                            keyword.to_owned(),
                            "Can't return a value from an initializer.".to_string(),
                        ));
                    }
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
            Expr::Get { object, name: _ } => self.resolve_expr(object),
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
            Expr::Set {
                object,
                name: _,
                value,
            } => {
                self.resolve_expr(object)?;
                self.resolve_expr(value)
            }
            Expr::Super {
                id,
                keyword,
                method: _,
            } => {
                match self.current_class {
                    ClassType::None => {
                        return Err(ResolverError::new(
                            keyword.to_owned(),
                            "Can't use 'super' outside of a class.".to_string(),
                        ))
                    }
                    ClassType::Class => {
                        return Err(ResolverError::new(
                            keyword.to_owned(),
                            "Can't user 'super' in a class with no superclass".to_string(),
                        ))
                    }
                    ClassType::Subclass => {}
                }
                self.resolve_local(id, keyword)
            }
            Expr::This { id, keyword } => match self.current_class {
                ClassType::None => Err(ResolverError::new(
                    keyword.to_owned(),
                    "Can't use 'this' outside of a class.".to_string(),
                )),
                ClassType::Class | ClassType::Subclass => self.resolve_local(id, keyword),
            },
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

    fn resolve_function(&mut self, func: &Function, func_type: FunctionType) -> ResolverResult {
        let enclosing_function = self.current_function;
        self.current_function = func_type;
        self.begin_scope();
        for param in &func.params {
            self.declare(param)?;
            self.define(param);
        }
        self.resolve_stmts(&func.body)?;
        self.end_scope();
        self.current_function = enclosing_function;
        Ok(())
    }

    fn resolve_local(&mut self, id: &usize, name: &Token) -> ResolverResult {
        for (depth, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter.resolve(id, depth)?;
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
