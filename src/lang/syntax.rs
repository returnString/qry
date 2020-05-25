use std::collections::HashMap;

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

impl BinaryOperator {
	pub fn name(self) -> Option<&'static str> {
		match self {
			Self::Add => Some("add"),
			Self::Sub => Some("sub"),
			Self::Mul => Some("mul"),
			Self::Div => Some("div"),
			Self::Access => Some("access"),
			Self::Equal => Some("equal"),
			Self::NotEqual => Some("notequal"),
			Self::Lt => Some("lte"),
			Self::Lte => Some("lte"),
			Self::Gt => Some("gt"),
			Self::Gte => Some("gte"),
			Self::And => Some("and"),
			Self::Or => Some("or"),
			Self::LAssign | Self::RAssign | Self::Pipe => None,
		}
	}
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
pub struct ParameterDef {
	pub name: String,
	pub param_type: Syntax,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwitchCase {
	pub expr: Syntax,
	pub returns: Syntax,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Syntax {
	Null,
	Int(i64),
	Float(f64),
	Bool(bool),
	String(String),
	Ident(String),
	Interpolate(Box<Syntax>),
	Use {
		from: Vec<String>,
		import: Import,
	},
	BinaryOp {
		op: BinaryOperator,
		lhs: Box<Syntax>,
		rhs: Box<Syntax>,
	},
	UnaryOp {
		op: UnaryOperator,
		target: Box<Syntax>,
	},
	Function {
		name: Option<String>,
		params: Vec<ParameterDef>,
		return_type: Box<Syntax>,
		body: Vec<Syntax>,
	},
	Call {
		target: Box<Syntax>,
		positional_args: Vec<Syntax>,
		named_args: HashMap<String, Syntax>,
	},
	Switch {
		target: Box<Syntax>,
		cases: Vec<SwitchCase>,
	},
}
