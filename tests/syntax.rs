use qry::runtime::{EvalError, Value};

pub mod helpers;

#[test]
fn test_syntax() {
	helpers::eval_expect_values(&[
		("1", Value::Int(1)),
		("4.5", Value::Float(4.5)),
		("null", Value::Null(())),
		("true", Value::Bool(true)),
		("true & true", Value::Bool(true)),
		("true & false", Value::Bool(false)),
		("false & false", Value::Bool(false)),
		("true | true", Value::Bool(true)),
		("true | false", Value::Bool(true)),
		("false | false", Value::Bool(false)),
		("false", Value::Bool(false)),
		("\"mystr\"", Value::String("mystr".into())),
		(
			"\"string with spaces\"",
			Value::String("string with spaces".into()),
		),
		("\"äççéñt\"", Value::String("äççéñt".into())),
		("\"😂\"", Value::String("😂".into())),
		("x <- y <- 0", Value::Int(0)),
		("0 -> y -> x", Value::Int(0)),
	]);
}

#[test]
fn test_syntax_failures() {
	helpers::eval_expect_errors(&[("x", EvalError::NotFound("x".to_string()))]);
}
