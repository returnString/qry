use std::panic::Location;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOperator {
	Add,
	Sub,
	Mul,
	Div,
	LAssign,
	RAssign,
	Access,
	Equal,
	NotEqual,
	Lt,
	Lte,
	Gt,
	Gte,
	Pipe,
	And,
	Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOperator {
	Negate,
	Minus,
}

impl UnaryOperator {
	pub fn name(self) -> Option<&'static str> {
		match self {
			Self::Negate => Some("negate"),
			Self::Minus => Some("minus"),
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum Import {
	Wildcard,
	Named(Vec<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParameterDef<T> {
	pub name: String,
	pub param_type: T,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwitchCase<T> {
	pub expr: T,
	pub returns: T,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SyntaxTree<T> {
	Null,
	Int(i64),
	Float(f64),
	Bool(bool),
	String(String),
	Ident(String),
	Interpolate(Box<T>),
	Use {
		from: Vec<String>,
		import: Import,
	},
	BinaryOp {
		op: BinaryOperator,
		lhs: Box<T>,
		rhs: Box<T>,
	},
	UnaryOp {
		op: UnaryOperator,
		target: Box<T>,
	},
	Function {
		name: Option<String>,
		params: Vec<ParameterDef<T>>,
		return_type: Box<T>,
		body: Vec<T>,
	},
	Call {
		target: Box<T>,
		positional_args: Vec<T>,
		named_args: Vec<(String, T)>,
	},
	Switch {
		target: Box<T>,
		cases: Vec<SwitchCase<T>>,
	},
	Index {
		target: Box<T>,
		keys: Vec<T>,
	},
}

#[derive(Debug, Clone, PartialEq)]
pub enum SourceLocation {
	User { line: usize, file: Rc<str> },
	Native { line: usize, file: Rc<str> },
	Unknown,
}

impl From<&Location<'_>> for SourceLocation {
	fn from(loc: &Location) -> Self {
		SourceLocation::Native {
			line: loc.line() as usize,
			file: loc.file().into(),
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxNode {
	pub syntax: SyntaxTree<Self>,
	pub location: SourceLocation,
}

pub type Syntax = SyntaxTree<SyntaxNode>;
