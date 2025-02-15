use super::{
    error::RuntimeError,
    expr::Expr,
    scanner::{Literal, TokenType},
    stmt::Stmt,
    value::Value,
};

pub type RuntimeResult<'a, T> = Result<T, RuntimeError<'a>>;

pub struct Interpreter;

impl Interpreter {
    pub fn interpret<'a>(&self, stmts: &'a [Stmt]) -> RuntimeResult<'a, ()> {
        for stmt in stmts {
            self.execute(stmt)?;
        }
        Ok(())
    }

    fn execute<'a>(&self, stmt: &'a Stmt) -> RuntimeResult<'a, ()> {
        match stmt {
            Stmt::Print { expr } => {
                let value = self.eval(expr)?;
                println!("{value}");
                Ok(())
            }
            Stmt::Expression { expr } => self.eval(expr).map(|_| ()),
        }
    }

    fn eval<'a>(&self, expr: &'a Expr) -> RuntimeResult<'a, Value> {
        match expr {
            Expr::Literal(lit) => Ok(match lit {
                Literal::Bool(b) => Value::Boolean(*b),
                Literal::Number(n) => Value::Number(*n),
                Literal::String(s) => Value::String(s.to_string()),
                Literal::Nil => Value::Nil,
            }),
            Expr::Grouping(expr) => self.eval(expr),
            Expr::Unary { operator, right } => {
                let right = self.eval(right)?;
                match operator.t_type {
                    TokenType::Bang => Ok(!right.is_truthy()),
                    TokenType::Minus => right.checked_negate(operator),
                    _ => unreachable!("Invalid Unary expression: {expr}"),
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.eval(left)?;
                let right = self.eval(right)?;
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
        }
    }
}
