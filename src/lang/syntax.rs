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
pub struct ParameterDef {
	pub name: String,
	pub param_type: SyntaxNode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwitchCase {
	pub expr: SyntaxNode,
	pub returns: SyntaxNode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Syntax {
	Null,
	Int(i64),
	Float(f64),
	Bool(bool),
	String(String),
	Ident(String),
	Interpolate(Box<SyntaxNode>),
	Use {
		from: Vec<String>,
		import: Import,
	},
	BinaryOp {
		op: BinaryOperator,
		lhs: Box<SyntaxNode>,
		rhs: Box<SyntaxNode>,
	},
	UnaryOp {
		op: UnaryOperator,
		target: Box<SyntaxNode>,
	},
	Function {
		name: Option<String>,
		params: Vec<ParameterDef>,
		return_type: Box<SyntaxNode>,
		body: Vec<SyntaxNode>,
	},
	Call {
		target: Box<SyntaxNode>,
		positional_args: Vec<SyntaxNode>,
		named_args: Vec<(String, SyntaxNode)>,
	},
	Switch {
		target: Box<SyntaxNode>,
		cases: Vec<SwitchCase>,
	},
	Index {
		target: Box<SyntaxNode>,
		keys: Vec<SyntaxNode>,
	},
}

#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxNode {
	pub syntax: Syntax,
	pub start_pos: usize,
	pub end_pos: usize,
}
