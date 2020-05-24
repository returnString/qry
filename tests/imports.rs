use qry::runtime::{EvalError, Type, Value};

pub mod helpers;

#[test]
fn test_imports() {
	helpers::eval_expect_values(&[
		("use core::{Int} Int", Value::Type(Type::Int)),
		("use core core::Int", Value::Type(Type::Int)),
	]);
}

#[test]
fn test_import_failures() {
	helpers::eval_expect_errors(&[
		("use core::{blah}", EvalError::NotFound("blah".into())),
		(
			"use nonexistentmodule",
			EvalError::NotFound("nonexistentmodule".into()),
		),
	]);
}
