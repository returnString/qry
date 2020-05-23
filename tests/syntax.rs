use qry::runtime::{InterpreterError, Value};

mod helpers;

#[test]
fn test_syntax() {
	helpers::eval_expect_values(&[
		("1", Value::Int(1)),
		("4.5", Value::Float(4.5)),
		("null", Value::Null),
		("true", Value::Bool(true)),
		("false", Value::Bool(false)),
		("\"mystr\"", Value::String("mystr".into())),
		("x <- y <- 0", Value::Int(0)),
		("0 -> y -> x", Value::Int(0)),
	]);
}

#[test]
fn test_syntax_failures() {
	helpers::eval_expect_errors(&[("x", InterpreterError::NotFound("x".to_string()))]);
}
