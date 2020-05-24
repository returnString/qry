use super::{SqlError, SqlMetadata, SqlResult};
use crate::lang::{BinaryOperator, Syntax};
use crate::runtime::{eval, EvalContext, Type, Value};
use crate::stdlib::ops::BINOP_LOOKUP;

#[derive(Clone, Debug)]
pub struct SqlExpression {
	pub sql_type: Type,
	pub text: String,
}

fn binop_symbol(op: &BinaryOperator) -> &'static str {
	match op {
		BinaryOperator::Add => "+",
		BinaryOperator::Sub => "-",
		BinaryOperator::Mul => "*",
		BinaryOperator::Div => "/",
		BinaryOperator::Equal => "==",
		BinaryOperator::NotEqual => "<>",
		BinaryOperator::Lt => "<",
		BinaryOperator::Lte => "<=",
		BinaryOperator::Gt => ">",
		BinaryOperator::Gte => ">=",
		BinaryOperator::LAssign | BinaryOperator::RAssign | BinaryOperator::Access => {
			panic!("invalid op for sql")
		}
	}
}

fn null_literal() -> SqlExpression {
	SqlExpression {
		sql_type: Type::Null,
		text: "null".into(),
	}
}

fn string_literal(s: &str) -> SqlExpression {
	SqlExpression {
		sql_type: Type::String,
		text: format!("'{}'", s),
	}
}

fn float_literal(f: f64) -> SqlExpression {
	SqlExpression {
		sql_type: Type::Float,
		text: format!("{:?}", f),
	}
}

fn int_literal(i: i64) -> SqlExpression {
	SqlExpression {
		sql_type: Type::Int,
		text: i.to_string(),
	}
}

fn bool_literal(b: bool) -> SqlExpression {
	SqlExpression {
		sql_type: Type::Bool,
		text: b.to_string(),
	}
}

fn interpret_value(val: Value) -> SqlExpression {
	match val {
		Value::Null(_) => null_literal(),
		Value::String(s) => string_literal(&s),
		Value::Int(i) => int_literal(i),
		Value::Float(f) => float_literal(f),
		Value::Bool(b) => bool_literal(b),
		_ => unreachable!(),
	}
}

pub fn expr_to_sql(
	ctx: &EvalContext,
	expr: &Syntax,
	metadata: &SqlMetadata,
) -> SqlResult<SqlExpression> {
	match expr {
		Syntax::Interpolate(contained_expr) => {
			Ok(interpret_value(eval(ctx, &[(**contained_expr).clone()])?))
		}
		Syntax::Null => Ok(null_literal()),
		Syntax::String(s) => Ok(string_literal(&s)),
		Syntax::Int(i) => Ok(int_literal(*i)),
		Syntax::Float(f) => Ok(float_literal(*f)),
		Syntax::Bool(b) => Ok(bool_literal(*b)),
		// FIXME: add proper types for columns when we started needing proper op dispatch
		Syntax::Ident(col_name) => Ok(SqlExpression {
			text: col_name.to_string(),
			sql_type: Type::String,
		}),
		Syntax::BinaryOp { lhs, op, rhs } => {
			let lhs_val = expr_to_sql(ctx, lhs, metadata)?;
			let rhs_val = expr_to_sql(ctx, rhs, metadata)?;
			match BINOP_LOOKUP.with(|b| b.get(&op).cloned()) {
				Some(method) => {
					let resolved = method
						.borrow()
						.resolve(&[lhs_val.sql_type, rhs_val.sql_type]);

					match resolved {
						Some(callable) => Ok(SqlExpression {
							text: format!("{} {} {}", lhs_val.text, binop_symbol(op), rhs_val.text),
							sql_type: callable.signature().return_type.clone(),
						}),
						None => Err(SqlError::SyntaxError),
					}
				}
				None => Err(SqlError::SyntaxError),
			}
		}
		_ => Err(SqlError::SyntaxError),
	}
}