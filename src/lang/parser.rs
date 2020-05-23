use crate::lang::syntax::*;
use std::collections::HashMap;

fn unop(target: Syntax, op: UnaryOperator) -> Syntax {
	Syntax::UnaryOp {
		op,
		target: Box::new(target),
	}
}

fn binop(lhs: Syntax, rhs: Syntax, op: BinaryOperator) -> Syntax {
	Syntax::BinaryOp {
		op,
		lhs: Box::new(lhs),
		rhs: Box::new(rhs),
	}
}

peg::parser! {
	grammar parser() for str {
		rule _()
			= [' ' | '\n' | '\t' | '\r']*

		rule ident() -> String
			= s:$(['a'..='z' | 'A'..='Z' | '_']+) { s.to_string() }

		rule param_def() -> ParameterDef
			= name:ident() _ ":" _ param_type:expr() { ParameterDef { name, param_type } }

		rule import_wildcard() -> Import
			= "::*" { Import::Wildcard }

		rule import_named() -> Import
			= "::{" _ names:ident() ** ("," _) _ "}" { Import::Named(names) }

		rule import_lib() -> Import
			= name:ident() { Import::Named(vec![name]) }

		rule expr() -> Syntax = precedence!{
			lhs:@ _ "<-" _ rhs:(@) { binop(lhs, rhs, BinaryOperator::LAssign) }
			--
			lhs:(@) _ "->" _ rhs:@ { binop(lhs, rhs, BinaryOperator::RAssign) }
			--
			"!" _ target:@ { unop(target, UnaryOperator::Negate) }
			--
			lhs:(@) _ "==" _ rhs:@ { binop(lhs, rhs, BinaryOperator::Equal) }
			lhs:(@) _ "!=" _ rhs:@ { binop(lhs, rhs, BinaryOperator::NotEqual) }
			lhs:(@) _ ">" _ rhs:@ { binop(lhs, rhs, BinaryOperator::Gt) }
			lhs:(@) _ ">=" _ rhs:@ { binop(lhs, rhs, BinaryOperator::Gte) }
			lhs:(@) _ "<" _ rhs:@ { binop(lhs, rhs, BinaryOperator::Lt) }
			lhs:(@) _ "<=" _ rhs:@ { binop(lhs, rhs, BinaryOperator::Lte) }
			--
			lhs:(@) _ "+" _ rhs:@ { binop(lhs, rhs, BinaryOperator::Add) }
			lhs:(@) _ "-" _ rhs:@ { binop(lhs, rhs, BinaryOperator::Sub) }
			--
			lhs:(@) _ "*" _ rhs:@ { binop(lhs, rhs, BinaryOperator::Mul) }
			lhs:(@) _ "/" _ rhs:@ { binop(lhs, rhs, BinaryOperator::Div) }
			--
			"fn" _ name:ident()? "(" _ params:param_def() ** (_ "," _) ")" _ "->" _ return_type:expr() _ "{" _ body:expr()* _ "}"  {
				Syntax::Function {
					name,
					params,
					body,
					return_type: Box::new(return_type)
				}
			}
			--
			target:@ "(" _ args:expr() ** (_ "," _) ")" {
				Syntax::Call {
					target: Box::new(target),
					positional_args: args,
					named_args: HashMap::new(),
				}
			}
			--
			"-" _ target:@ { unop(target, UnaryOperator::Minus) }
			--
			lhs:(@) "::" rhs:@ { binop(lhs, rhs, BinaryOperator::Access) }
			--
			"use" _ from:ident() ** "::" import:(import_named() / import_wildcard() / import_lib()) { Syntax::Use { from, import } }
			"use" _ import:import_lib() { Syntax::Use { from: vec![], import } }
			--
			n:$(['0'..='9']+ "." ['0'..='9']*) { Syntax::Float(n.parse().unwrap()) }
			n:$(['0'..='9']+) { Syntax::Int(n.parse().unwrap()) }
			"\"" s:$(['a'..='z' | 'A'..='Z']+) "\"" { Syntax::String(s.to_string()) }
			b:$("true" / "false") { Syntax::Bool(b == "true") }
			"null" { Syntax::Null }
			ident:ident() { Syntax::Ident(ident) }
			"(" e:expr() ")" { e }
		}

		pub rule program() -> Vec<Syntax>
			= exprs:expr() ** _ { exprs }
	}
}

pub fn parse(src: &str) -> Result<Vec<Syntax>, peg::error::ParseError<peg::str::LineCol>> {
	parser::program(src)
}
