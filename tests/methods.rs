use qry::runtime::{InterpreterError, Value};

pub mod helpers;

#[test]
fn test_methods() {
	helpers::eval_expect_values(&[
		("use ops ops::add(ops::add(1, 2), 10)", Value::Int(13)),
		("1 + 2 + 10", Value::Int(13)),
		("\"hai\" + \"world\"", Value::String("haiworld".into())),
		("9 / 2", Value::Int(4)),
		("9 / 2.", Value::Float(4.5)),
		("1 + 2", Value::Int(3)),
		("1 + 2 * 3", Value::Int(7)),
		("(1 + 2) * 3", Value::Int(9)),
		("!true", Value::Bool(false)),
		("!false", Value::Bool(true)),
		("0 > 1", Value::Bool(false)),
		("0 < 1", Value::Bool(true)),
		("0 >= 1", Value::Bool(false)),
		("0 <= 1", Value::Bool(true)),
		("10.1 > 10", Value::Bool(true)),
		("-1", Value::Int(-1)),
		("-10.5", Value::Float(-10.5)),
		("(\"hai\" + \"world\") == \"haiworld\"", Value::Bool(true)),
		("(\"hai\" + \"world\") == \"bye\"", Value::Bool(false)),
		("(\"hai\" + \"world\") != \"haiworld\"", Value::Bool(false)),
		("to_string(10)", Value::String("10".into())),
		("to_string(1.5)", Value::String("1.5".into())),
		("to_string(true)", Value::String("true".into())),
		("to_string(false)", Value::String("false".into())),
		("to_string(null)", Value::String("null".into())),
	]);
}

#[test]
fn test_method_failures() {
	helpers::eval_expect_errors(&[
		(
			"use ops ops::add(null, null)",
			InterpreterError::MethodNotImplemented,
		),
		("null + null", InterpreterError::MethodNotImplemented),
	]);
}
