use super::{
    environment::Environment,
    error::RuntimeError,
    expr::Expr,
    scanner::{Literal, TokenType},
    stmt::Stmt,
    value::Value,
};

pub type RuntimeResult<'a, T> = Result<T, RuntimeError<'a>>;

#[derive(Default)]
pub struct Interpreter {
    environment: Box<Environment>,
}

impl Interpreter {
    pub fn interpret<'a>(&mut self, stmts: &'a [Stmt]) -> RuntimeResult<'a, ()> {
        for stmt in stmts {
            self.execute(stmt)?;
        }
        Ok(())
    }

    fn execute<'a>(&mut self, stmt: &'a Stmt) -> RuntimeResult<'a, ()> {
        match stmt {
            Stmt::Block(statements) => {
                self.execute_block(statements)?;
            }
            Stmt::Print { expr } => {
                let value = self.evaluate(expr)?;
                println!("{value}");
            }
            Stmt::Expression { expr } => {
                self.evaluate(expr)?;
            }
            Stmt::Var { name, initializer } => {
                let value = self.evaluate(initializer)?;
                self.environment.define(&name.lexeme, value);
            }
        }
        Ok(())
    }

    fn execute_block<'a>(&mut self, statements: &'a [Stmt]) -> RuntimeResult<'a, ()> {
        let new_environment = Box::new(Environment::default());
        let previous = std::mem::replace(&mut self.environment, new_environment);
        self.environment.enclosing = Some(previous);
        for stmt in statements {
            self.execute(stmt)
                .inspect_err(|_| self.environment = self.environment.enclosing.take().unwrap())?;
        }
        self.environment = self.environment.enclosing.take().unwrap();
        Ok(())
    }

    fn evaluate<'a>(&mut self, expr: &'a Expr) -> RuntimeResult<'a, Value> {
        match expr {
            Expr::Assign { name, value } => {
                let value = self.evaluate(value)?;
                self.environment.assign(name, value.clone())?;
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
                    TokenType::BangEqual => Ok(Value::Boolean(left != right)),
                    TokenType::EqualEqual => Ok(Value::Boolean(left == right)),
                    _ => unreachable!("Invalid Binary expression: {expr}"),
                }
            }
            Expr::Grouping(expr) => self.evaluate(expr),
            Expr::Literal(lit) => Ok(match lit {
                Literal::Bool(b) => Value::Boolean(*b),
                Literal::Number(n) => Value::Number(*n),
                Literal::String(s) => Value::String(s.to_string()),
                Literal::Nil => Value::Nil,
            }),
            Expr::Unary { operator, right } => {
                let right = self.evaluate(right)?;
                match operator.t_type {
                    TokenType::Bang => Ok(!right.is_truthy()),
                    TokenType::Minus => right.checked_negate(operator),
                    _ => unreachable!("Invalid Unary expression: {expr}"),
                }
            }
            Expr::Variable(token) => self.environment.get(token),
        }
    }
}
