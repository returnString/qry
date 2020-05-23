use qry::runtime::{InterpreterError, Value};

mod helpers;

#[test]
fn test_methods() {
	helpers::eval_expect_values(&[
		("add(add(1, 2), 10)", Value::Int(13)),
		("1 + 2 + 10", Value::Int(13)),
		("\"hai\" + \"world\"", Value::String("haiworld".into())),
		("9 / 2", Value::Int(4)),
		("9 / 2.", Value::Float(4.5)),
	]);
}

#[test]
fn test_method_failures() {
	helpers::eval_expect_errors(&[
		("add(null, null)", InterpreterError::MethodNotImplemented),
		("null + null", InterpreterError::MethodNotImplemented),
	]);
}
