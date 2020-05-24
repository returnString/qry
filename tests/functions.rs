use qry::lang::parse;
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
			"fn getter(a: Int) -> Null { fn() -> Int { a + 1 } }
			getter(30)()",
			Value::Int(31),
		),
		(
			"parse(x + 1)",
			Value::Syntax(Box::new(parse("x + 1").unwrap()[0].clone())),
		),
	]);
}

#[test]
fn test_function_failures() {
	helpers::eval_expect_errors(&[
		("1()", EvalError::NotCallable),
		("typeof(1, 2)", EvalError::ArgMismatch),
	]);
}
