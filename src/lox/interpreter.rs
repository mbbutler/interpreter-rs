use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{SystemTime, UNIX_EPOCH},
};

use super::{
    environment::Environment,
    error::RuntimeException,
    expr::Expr,
    lox_callable::{LoxCallable, LoxCallableFn},
    resolver::ResolverResult,
    scanner::{Token, TokenType},
    stmt::Stmt,
    value::Value,
};

pub type RuntimeResult<T> = Result<T, RuntimeException>;

#[derive(Default)]
pub struct Interpreter {
    #[allow(unused)]
    pub globals: Arc<RwLock<Environment>>,
    pub environment: Arc<RwLock<Environment>>,
    pub locals: HashMap<usize, usize>,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Arc::new(RwLock::new(Environment::default()));
        globals.write().unwrap().define(
            "clock",
            Value::Callable(LoxCallable::new_native(0, |_, _| {
                Ok(Value::Number(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as f64,
                ))
            })),
        );
        let environment = Arc::clone(&globals);
        Self {
            globals,
            environment,
            locals: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, stmts: &[Stmt]) -> RuntimeResult<()> {
        for stmt in stmts {
            self.execute(stmt)?;
        }
        Ok(())
    }

    fn execute(&mut self, stmt: &Stmt) -> RuntimeResult<()> {
        match stmt {
            Stmt::Block(statements) => {
                let new_environment = Environment::new(&self.environment);
                self.execute_block(statements, new_environment)?;
            }
            Stmt::Expression(expr) => {
                self.evaluate(expr)?;
            }
            Stmt::Function { name, params, body } => {
                let function = Value::Callable(LoxCallable::new_lox(
                    Box::new(name.to_owned()),
                    params.clone(),
                    body.clone(),
                    &self.environment,
                ));
                self.environment.write()?.define(&name.lexeme, function);
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if self.evaluate(condition)?.is_truthy() {
                    self.execute(then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.execute(else_branch)?;
                }
            }
            Stmt::Print(expr) => {
                let value = self.evaluate(expr)?;
                println!("{value}");
            }
            Stmt::Return { keyword: _, value } => {
                let value = match value {
                    Some(expr) => self.evaluate(expr)?,
                    None => Value::Nil,
                };
                return Err(RuntimeException::new_return(value));
            }
            Stmt::Var { name, initializer } => {
                let value = if let Some(initializer) = initializer {
                    self.evaluate(initializer)?
                } else {
                    Value::Nil
                };
                self.environment.write()?.define(&name.lexeme, value);
            }
            Stmt::While { condition, body } => {
                while self.evaluate(condition)?.is_truthy() {
                    self.execute(body)?;
                }
            }
        }
        Ok(())
    }

    pub fn execute_block(
        &mut self,
        statements: &[Stmt],
        environment: Arc<RwLock<Environment>>,
    ) -> RuntimeResult<()> {
        let previous = Arc::clone(&self.environment);
        self.environment = environment;
        for stmt in statements {
            self.execute(stmt).inspect_err(|_| {
                self.environment = Arc::clone(&previous);
            })?;
        }
        self.environment = previous;
        Ok(())
    }

    pub fn resolve(&mut self, id: &usize, depth: usize) -> ResolverResult {
        self.locals.insert(*id, depth);
        Ok(())
    }

    fn evaluate(&mut self, expr: &Expr) -> RuntimeResult<Value> {
        match expr {
            Expr::Assign {
                id,
                name,
                value: value_expr,
            } => {
                let value = self.evaluate(value_expr)?;
                match self.locals.get(id) {
                    Some(distance) => {
                        self.environment
                            .write()?
                            .assign_at(*distance, name, value.clone())?
                    }
                    None => self.globals.write()?.assign(name, value.clone())?,
                }
                Ok(value)
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;
                match operator.t_type {
                    TokenType::Plus => left.checked_add(operator, &right),
                    TokenType::Minus => left.checked_sub(operator, &right),
                    TokenType::Slash => left.checked_div(operator, &right),
                    TokenType::Star => left.checked_mul(operator, &right),
                    TokenType::Greater => left.checked_gt(operator, &right),
                    TokenType::GreaterEqual => left.checked_gte(operator, &right),
                    TokenType::Less => left.checked_lt(operator, &right),
                    TokenType::LessEqual => left.checked_lte(operator, &right),
                    TokenType::BangEqual => Ok(Value::Bool(left != right)),
                    TokenType::EqualEqual => Ok(Value::Bool(left == right)),
                    _ => unreachable!("Invalid Binary expression: {expr}"),
                }
            }
            Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                let callee = self.evaluate(callee)?;
                let mut args = Vec::new();
                for argument in arguments {
                    args.push(self.evaluate(argument)?);
                }

                if let Value::Callable(callee) = callee {
                    if args.len() != callee.arity() {
                        Err(RuntimeException::new_error(
                            paren.to_owned(),
                            format!(
                                "Expected {} arguments but got {}.",
                                callee.arity(),
                                args.len()
                            ),
                        ))
                    } else {
                        callee.call(self, &args)
                    }
                } else {
                    Err(RuntimeException::new_error(
                        paren.to_owned(),
                        "Can only call functions and classes.".to_string(),
                    ))
                }
            }
            Expr::Grouping(expr) => self.evaluate(expr),
            Expr::Literal(value) => Ok(value.to_owned()),
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(left)?;
                if let TokenType::Or = operator.t_type {
                    if left.is_truthy() {
                        return Ok(left);
                    }
                } else if !left.is_truthy() {
                    return Ok(left);
                }
                self.evaluate(right)
            }
            Expr::Unary { operator, right } => {
                let right = self.evaluate(right)?;
                match operator.t_type {
                    TokenType::Bang => Ok(right.not()),
                    TokenType::Minus => right.checked_negate(operator),
                    _ => unreachable!("Invalid Unary expression: {expr}"),
                }
            }
            Expr::Variable { id, name } => self.look_up_var(name, id),
        }
    }

    fn look_up_var(&self, name: &Token, id: &usize) -> RuntimeResult<Value> {
        match self.locals.get(id) {
            Some(distance) => self.environment.read()?.get_at(*distance, name),
            None => self.globals.read()?.get(name),
        }
    }
}
