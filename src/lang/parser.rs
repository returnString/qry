use crate::lang::syntax::*;

fn unop(target: RawSyntaxNode, op: UnaryOperator) -> SyntaxTree<RawSyntaxNode> {
	SyntaxTree::UnaryOp {
		op,
		target: Box::new(target),
	}
}

fn binop(lhs: RawSyntaxNode, rhs: RawSyntaxNode, op: BinaryOperator) -> SyntaxTree<RawSyntaxNode> {
	SyntaxTree::BinaryOp {
		op,
		lhs: Box::new(lhs),
		rhs: Box::new(rhs),
	}
}

peg::parser! {
	grammar parser() for str {
		rule _()
			= [' ' | '\n' | '\t' | '\r']*

		rule __()
			= [' ' | '\n' | '\t' | '\r']+

		rule ident() -> String
			= s:$(['a'..='z' | 'A'..='Z' | '_']+) { s.to_string() }

		rule param_def() -> ParameterDef<RawSyntaxNode>
			= name:ident() _ ":" _ param_type:expr() { ParameterDef { name, param_type } }

		rule import_wildcard() -> Import
			= "::*" { Import::Wildcard }

		rule import_named() -> Import
			= "::{" _ names:ident() ** ("," _) _ "}" { Import::Named(names) }

		rule import_lib() -> Import
			= name:ident() { Import::Named(vec![name]) }

		rule fn_named_prefix() -> Option<String>
			= "fn" __ name:ident() { Some(name) }

		rule fn_anon_prefix() -> Option<String>
			= "fn" { None }

		rule named_arg() -> (Option<String>, RawSyntaxNode)
			= name:ident() _ "=" _ expr:expr() { (Some(name), expr) }

		rule positional_arg() -> (Option<String>, RawSyntaxNode)
			= expr:expr() { (None, expr) }

		// FIXME: add char escapes
		rule string_contents() -> String
			= s:$(!"\"" [_])* { s.iter().fold("".to_string(), |acc, val| format!("{}{}", acc, val)) }

		rule switch_case() -> SwitchCase<RawSyntaxNode>
			= expr:expr() _ "=>" _ returns:expr() { SwitchCase { expr, returns } }

		rule expr() -> RawSyntaxNode = precedence!{
			start_pos:position!() syntax:@ end_pos:position!() { RawSyntaxNode { start_pos, end_pos, syntax } }
			--
			lhs:@ _ "<-" _ rhs:(@) { binop(lhs, rhs, BinaryOperator::LAssign) }
			--
			lhs:(@) _ "->" _ rhs:@ { binop(lhs, rhs, BinaryOperator::RAssign) }
			--
			lhs:(@) _ "|" _ rhs:@ { binop(lhs, rhs, BinaryOperator::Or) }
			--
			lhs:(@) _ "&" _ rhs:@ { binop(lhs, rhs, BinaryOperator::And) }
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
			name:(fn_named_prefix() / fn_anon_prefix()) _ "(" _ params:param_def() ** (_ "," _) ")" _ "->" _ return_type:expr() _ "{" _ body:expr()* _ "}"  {
				SyntaxTree::Function {
					name,
					params,
					body,
					return_type: Box::new(return_type)
				}
			}
			--
			lhs:(@) _ "|>" _ rhs:@ { binop(lhs, rhs, BinaryOperator::Pipe) }
			--
			"-" _ target:@ { unop(target, UnaryOperator::Minus) }
			--
			target:@ "(" _ args:(named_arg() / positional_arg()) ** (_ "," _) ")" {
				SyntaxTree::Call {
					target: Box::new(target),
					positional_args: args.iter().filter(|(n, e)| n.is_none()).map(|(n, e)| e.clone()).collect(),
					named_args: args.iter().filter(|(n, e)| n.is_some()).map(|(n, e)| (n.clone().unwrap(), e.clone())).collect(),
				}
			}
			--
			target:(@) "[" keys:expr() ** (_ "," _) "]" { SyntaxTree::Index { target: Box::new(target), keys } }
			--
			lhs:(@) "::" rhs:@ { binop(lhs, rhs, BinaryOperator::Access) }
			--
			"use" __ from:ident() ** "::" import:(import_named() / import_wildcard() / import_lib()) { SyntaxTree::Use { from, import } }
			"use" __ import:import_lib() { SyntaxTree::Use { from: vec![], import } }
			--
			"switch" _ target:expr() _ "{" _ cases:switch_case() ** _ _ "}" { SyntaxTree::Switch { target: Box::new(target), cases } }
			--
			n:
			$(['0'..='9']+ "." ['0'..='9']*) { SyntaxTree::Float(n.parse().unwrap()) }
			n:$(['0'..='9']+) { SyntaxTree::Int(n.parse().unwrap()) }
			"\"" s:string_contents() "\"" { SyntaxTree::String(s) }
			b:$("true" / "false") { SyntaxTree::Bool(b == "true") }
			"null" { SyntaxTree::Null }
			ident:ident() { SyntaxTree::Ident(ident) }
			"(" _ e:expr() _ ")" { e.syntax }
			"{{" _ e:expr() _ "}}" { SyntaxTree::Interpolate(Box::new(e)) }
		}

		pub(in super) rule program() -> Vec<RawSyntaxNode>
			= _ exprs:expr() ** _ _ { exprs }
	}
}

#[derive(Debug, Clone, PartialEq)]
struct RawSyntaxNode {
	pub syntax: SyntaxTree<Self>,
	pub start_pos: usize,
	pub end_pos: usize,
}

struct SourceLocationMapper {
	linebreak_offsets: Vec<usize>,
}

impl SourceLocationMapper {
	fn map(&self, node: &RawSyntaxNode) -> Box<SyntaxNode> {
		let new_syntax = match &node.syntax {
			SyntaxTree::Call {
				target,
				positional_args,
				named_args,
			} => SyntaxTree::Call {
				target: self.map(&target),
				positional_args: positional_args.iter().map(|a| *self.map(&a)).collect(),
				named_args: named_args
					.iter()
					.map(|(n, a)| (n.clone(), *self.map(&a)))
					.collect(),
			},
			SyntaxTree::Int(v) => SyntaxTree::Int(*v),
			SyntaxTree::Float(v) => SyntaxTree::Float(*v),
			SyntaxTree::String(v) => SyntaxTree::String(v.clone()),
			SyntaxTree::Bool(v) => SyntaxTree::Bool(*v),
			SyntaxTree::Null => SyntaxTree::Null,
			SyntaxTree::Ident(n) => SyntaxTree::Ident(n.clone()),
			SyntaxTree::Interpolate(expr) => SyntaxTree::Interpolate(self.map(expr)),
			SyntaxTree::BinaryOp { op, lhs, rhs } => SyntaxTree::BinaryOp {
				op: *op,
				lhs: self.map(lhs),
				rhs: self.map(rhs),
			},
			SyntaxTree::UnaryOp { op, target } => SyntaxTree::UnaryOp {
				op: *op,
				target: self.map(target),
			},
			SyntaxTree::Use { from, import } => SyntaxTree::Use {
				from: from.clone(),
				import: import.clone(),
			},
			SyntaxTree::Function {
				name,
				params,
				return_type,
				body,
			} => SyntaxTree::Function {
				name: name.clone(),
				params: params
					.iter()
					.map(|p| ParameterDef {
						name: p.name.clone(),
						param_type: *self.map(&p.param_type),
					})
					.collect(),
				return_type: self.map(return_type),
				body: body.iter().map(|e| *self.map(e)).collect(),
			},
			SyntaxTree::Switch { target, cases } => SyntaxTree::Switch {
				target: self.map(target),
				cases: cases
					.iter()
					.map(|c| SwitchCase {
						expr: *self.map(&c.expr),
						returns: *self.map(&c.returns),
					})
					.collect(),
			},
			SyntaxTree::Index { target, keys } => SyntaxTree::Index {
				target: self.map(target),
				keys: keys.iter().map(|k| *self.map(k)).collect(),
			},
		};

		let line = match self.linebreak_offsets.binary_search(&node.start_pos) {
			Ok(l) => l,
			Err(l) => l,
		} + 1;

		Box::new(SyntaxNode {
			syntax: new_syntax,
			line,
		})
	}
}

fn linebreaks_from_source(src: &str) -> Vec<usize> {
	src
		.bytes()
		.enumerate()
		.filter_map(|(i, b)| match b {
			b'\n' => Some(i),
			_ => None,
		})
		.collect()
}

pub fn parse(src: &str) -> Result<Vec<SyntaxNode>, peg::error::ParseError<peg::str::LineCol>> {
	let raw_roots = parser::program(src)?;
	let src_mapper = SourceLocationMapper {
		linebreak_offsets: linebreaks_from_source(src),
	};
	Ok(raw_roots.iter().map(|r| *src_mapper.map(r)).collect())
}
