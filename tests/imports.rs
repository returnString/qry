use qry::runtime::{InterpreterError, Type, Value};

mod helpers;

#[test]
fn test_imports() {
	helpers::eval_expect_values(&[
		("use types::{Int} Int", Value::Type(Type::Int)),
		("use types types::Int", Value::Type(Type::Int)),
	]);
}

#[test]
fn test_import_failures() {
	helpers::eval_expect_errors(&[
		(
			"use types::{blah}",
			InterpreterError::NotFound("blah".into()),
		),
		(
			"use nonexistentmodule",
			InterpreterError::NotFound("nonexistentmodule".into()),
		),
	]);
}
