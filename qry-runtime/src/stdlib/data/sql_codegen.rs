use super::{ColumnMap, Vector};
use crate::{eval, EvalContext, EvalResult, NativeGenericType, Type, Value};
use qry_lang::{BinaryOperator, Syntax, SyntaxNode};

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
		BinaryOperator::And => "and",
		BinaryOperator::Or => "or",
		BinaryOperator::LAssign
		| BinaryOperator::RAssign
		| BinaryOperator::Access
		| BinaryOperator::Pipe => panic!("invalid op for sql"),
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
	expr: &SyntaxNode,
	metadata: &ColumnMap,
	as_aggregation: bool,
) -> EvalResult<SqlExpression> {
	match &expr.syntax {
		Syntax::Interpolate(contained_expr) => Ok(interpret_value(eval(ctx, contained_expr)?)),
		Syntax::Null => Ok(null_literal()),
		Syntax::String(s) => Ok(string_literal(&s)),
		Syntax::Int(i) => Ok(int_literal(*i)),
		Syntax::Float(f) => Ok(float_literal(*f)),
		Syntax::Bool(b) => Ok(bool_literal(*b)),
		Syntax::Ident(col_name) => Ok(SqlExpression {
			text: col_name.to_string(),
			sql_type: metadata[col_name].data_type.clone(),
		}),
		Syntax::BinaryOp { lhs, op, rhs } => {
			let lhs_val = expr_to_sql(ctx, lhs, metadata, as_aggregation)?;
			let rhs_val = expr_to_sql(ctx, rhs, metadata, as_aggregation)?;
			match ctx.methods.binops.get(&op) {
				Some(method) => {
					let resolved = method.resolve(&[lhs_val.sql_type, rhs_val.sql_type]);

					match resolved {
						Some(callable) => Ok(SqlExpression {
							text: format!("{} {} {}", lhs_val.text, binop_symbol(op), rhs_val.text),
							sql_type: callable.signature().return_type.clone(),
						}),
						None => Err(ctx.exception(&expr.location, "failed to resolve method")),
					}
				}
				None => Err(ctx.exception(&expr.location, "unhandled binary operator")),
			}
		}
		Syntax::Switch { target, cases } => {
			let target_val = expr_to_sql(ctx, target, metadata, as_aggregation)?;

			let whens = cases
				.iter()
				.map(|c| {
					let case_val = expr_to_sql(ctx, &c.expr, metadata, as_aggregation)?;
					let return_val = expr_to_sql(ctx, &c.returns, metadata, as_aggregation)?;
					Ok(format!(
						"when {} == {} then {}",
						target_val.text, case_val.text, return_val.text,
					))
				})
				.collect::<EvalResult<Vec<_>>>()?;

			let text = format!("case {} end", whens.join(" "));

			// TODO: validate consistency of return types
			// for now, just use the first
			let sql_type = expr_to_sql(ctx, &cases[0].returns, metadata, as_aggregation)?.sql_type;

			Ok(SqlExpression { text, sql_type })
		}
		Syntax::Call {
			target,
			positional_args,
			..
		} => {
			let target_ident = match &target.syntax {
				Syntax::Ident(n) => Ok(n),
				_ => Err(ctx.exception(&target.location, "expected identifier for call")),
			}?;

			let method = ctx
				.env
				.borrow()
				.get(&target_ident)
				.ok_or_else(|| ctx.exception(&target.location, "no method found"))?
				.as_method();

			let args = positional_args
				.iter()
				.map(|a| expr_to_sql(ctx, a, metadata, as_aggregation))
				.collect::<EvalResult<Vec<_>>>()?;

			let arg_text = args
				.iter()
				.map(|a| a.text.clone())
				.collect::<Vec<_>>()
				.join(", ");

			let arg_types = args
				.iter()
				.map(|a| {
					if as_aggregation {
						Ok(Vector::resolve(ctx, &[a.sql_type.clone()])?)
					} else {
						Ok(a.sql_type.clone())
					}
				})
				.collect::<EvalResult<Vec<_>>>()?;

			let text = format!("{}({})", method.name(), arg_text);

			let method_impl = method
				.resolve(&arg_types)
				.ok_or_else(|| ctx.exception(&expr.location, "failed to resolve method impl"))?;

			Ok(SqlExpression {
				text,
				sql_type: method_impl.signature().return_type.clone(),
			})
		}
		_ => Err(ctx.exception(&expr.location, "unhandled syntax")),
	}
}
