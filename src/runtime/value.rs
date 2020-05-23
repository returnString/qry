use super::{Builtin, EnvironmentPtr, Function, MethodPtr};
use crate::lang::Syntax;
use std::rc::Rc;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Type {
	Null,
	Int,
	Float,
	Bool,
	String,
	Type,
	Function,
	Builtin,
	Method,
	Library,
	Syntax,
	SyntaxPlaceholder,
	MethodDispatchPlaceholder,
}

#[derive(Debug, Clone)]
pub enum Value {
	Null,
	Int(i64),
	Float(f64),
	Bool(bool),
	String(Box<str>),
	Type(Type),
	Function(Rc<Function>),
	Builtin(Rc<Builtin>),
	Method(MethodPtr),
	Library(EnvironmentPtr),
	Syntax(Box<Syntax>),
}

impl Value {
	pub fn runtime_type(&self) -> Type {
		match *self {
			Self::Null => Type::Null,
			Self::Int(_) => Type::Int,
			Self::Float(_) => Type::Float,
			Self::Bool(_) => Type::Bool,
			Self::String(_) => Type::String,
			Self::Type(_) => Type::Type,
			Self::Function(_) => Type::Function,
			Self::Builtin(_) => Type::Builtin,
			Self::Method(_) => Type::Method,
			Self::Library(_) => Type::Library,
			Self::Syntax(_) => Type::Syntax,
		}
	}
}

impl PartialEq<Value> for Value {
	fn eq(&self, other: &Value) -> bool {
		match (self, &other) {
			(Value::Null, Value::Null) => true,
			(Value::Int(a), Value::Int(b)) => a == b,
			(Value::Float(a), Value::Float(b)) => a == b,
			(Value::Bool(a), Value::Bool(b)) => a == b,
			(Value::String(a), Value::String(b)) => a == b,
			(Value::Type(a), Value::Type(b)) => a == b,
			(Value::Function(a), Value::Function(b)) => Rc::ptr_eq(a, b),
			(Value::Builtin(a), Value::Builtin(b)) => Rc::ptr_eq(a, b),
			(Value::Method(a), Value::Method(b)) => Rc::ptr_eq(a, b),
			(Value::Library(a), Value::Library(b)) => Rc::ptr_eq(a, b),
			(Value::Syntax(a), Value::Syntax(b)) => a == b,
			_ => false,
		}
	}
}
