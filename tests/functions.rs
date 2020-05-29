use qry::runtime::{EvalError, Type, Value};

pub mod helpers;

#[test]
fn test_functions() {
	helpers::eval_expect_values(&[
		("typeof(1)", Value::Type(Type::Int)),
		("fn() -> Int { 10 }()", Value::Int(10)),
		(
			"myfunc <- fn(a: Int) -> Int { a }
			myfunc(20)",
			Value::Int(20),
		),
		(
			// TODO: syntax for function types
			"fn getter(a: Int) -> Any { fn() -> Int { a + 1 } }
			getter(30)()",
			Value::Int(31),
		),
		("list()", Value::List(vec![])),
		(
			r#"list(1, 2.0, "test string")"#,
			Value::List(vec![
				Value::Int(1),
				Value::Float(2.0),
				Value::String("test string".into()),
			]),
		),
		(r#"list(1, 2.0, "test string")[0]"#, Value::Int(1)),
		(r#"list(1, 2.0, "test string")[1]"#, Value::Float(2.)),
	]);
}

#[test]
fn test_function_failures() {
	helpers::eval_expect_errors(&[
		("1()", EvalError::NotCallable),
		("typeof(1, 2)", EvalError::ArgMismatch),
	]);
}
