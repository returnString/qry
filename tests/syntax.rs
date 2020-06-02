use qry::lang::{parse, BinaryOperator, SourceLocation, Syntax, SyntaxNode};
use qry::runtime::{EvalError, Value};

pub mod helpers;

#[test]
fn test_syntax() {
	helpers::eval_expect_values(&[
		("1", Value::Int(1)),
		("4.5", Value::Float(4.5)),
		("null", Value::Null(())),
		("true", Value::Bool(true)),
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
		(
			r#"switch 1 {
				1 => "got one"
			}"#,
			Value::String("got one".into()),
		),
		(
			r#"target <- 1
			switch target {
				2 => "got two"
				1 => "got one"
			}"#,
			Value::String("got one".into()),
		),
		(
			r#"
			switch 1 {
				2 => "nope"
				3 => "nope"
			}"#,
			Value::Null(()),
		),
	]);
}

#[test]
fn test_syntax_failures() {
	helpers::eval_expect_errors(&[("x", EvalError::NotFound("x".to_string()))]);
}

#[test]
fn test_syntax_locations() {
	let multiline_src = "test
x +
1
pipe
	|> into()
	|> something()
";

	let exprs = parse(multiline_src).unwrap();
	assert_eq!(exprs.len(), 3);

	assert_eq!(
		exprs[0],
		SyntaxNode {
			location: SourceLocation::User { line: 1 },
			syntax: Syntax::Ident("test".to_string()),
		}
	);

	assert_eq!(
		exprs[1],
		SyntaxNode {
			location: SourceLocation::User { line: 2 },
			syntax: Syntax::BinaryOp {
				op: BinaryOperator::Add,
				lhs: Box::new(SyntaxNode {
					location: SourceLocation::User { line: 2 },
					syntax: Syntax::Ident("x".to_string()),
				}),
				rhs: Box::new(SyntaxNode {
					location: SourceLocation::User { line: 3 },
					syntax: Syntax::Int(1)
				}),
			}
		}
	);

	assert_eq!(
		exprs[2],
		SyntaxNode {
			syntax: Syntax::BinaryOp {
				op: BinaryOperator::Pipe,
				lhs: Box::new(SyntaxNode {
					syntax: Syntax::BinaryOp {
						op: BinaryOperator::Pipe,
						lhs: Box::new(SyntaxNode {
							syntax: Syntax::Ident("pipe".to_string()),
							location: SourceLocation::User { line: 4 }
						}),
						rhs: Box::new(SyntaxNode {
							syntax: Syntax::Call {
								target: Box::new(SyntaxNode {
									syntax: Syntax::Ident("into".to_string()),
									location: SourceLocation::User { line: 5 }
								}),
								positional_args: vec![],
								named_args: vec![],
							},
							location: SourceLocation::User { line: 5 }
						}),
					},
					location: SourceLocation::User { line: 4 }
				}),
				rhs: Box::new(SyntaxNode {
					syntax: Syntax::Call {
						target: Box::new(SyntaxNode {
							syntax: Syntax::Ident("something".to_string()),
							location: SourceLocation::User { line: 6 }
						}),
						positional_args: vec![],
						named_args: vec![],
					},
					location: SourceLocation::User { line: 6 }
				}),
			},
			location: SourceLocation::User { line: 4 },
		}
	);
}
