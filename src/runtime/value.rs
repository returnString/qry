use super::{Builtin, EnvironmentPtr, Function, MethodPtr};
use crate::lang::Syntax;
use std::any::{Any, TypeId};
use std::rc::Rc;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct NativeDescriptor {
	id: TypeId,
	name: &'static str,
}

impl NativeDescriptor {
	pub fn of<T: 'static + NativeType>() -> NativeDescriptor {
		NativeDescriptor {
			id: TypeId::of::<T>(),
			name: T::name(),
		}
	}
}

pub trait NativeType {
	fn name() -> &'static str;
}

#[derive(Debug, Clone)]
pub struct NativeWrapper {
	descriptor: Box<NativeDescriptor>,
	obj: Rc<dyn Any>,
}

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
	Native(Box<NativeDescriptor>),
}

impl Type {
	pub fn new_native<T: 'static + NativeType>() -> Type {
		Type::Native(Box::new(NativeDescriptor::of::<T>()))
	}
}

#[derive(Debug, Clone)]
pub enum Value {
	Null(()),
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
	Native(NativeWrapper),
}

impl Value {
	pub fn runtime_type(&self) -> Type {
		match self {
			Self::Null(_) => Type::Null,
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
			Self::Native(w) => Type::Native(w.descriptor.clone()),
		}
	}

	pub fn as_native<T>(&self) -> Rc<T>
	where
		T: Any,
	{
		if let Self::Native(w) = self {
			return w.obj.clone().downcast::<T>().unwrap();
		}

		panic!("value is not a native type");
	}

	pub fn as_string(&self) -> &str {
		match self {
			Self::String(s) => s,
			_ => panic!("value is not a string"),
		}
	}

	pub fn as_syntax(&self) -> &Syntax {
		match self {
			Self::Syntax(expr) => expr,
			_ => panic!("value is not an expression"),
		}
	}

	pub fn as_bool(&self) -> bool {
		match self {
			Self::Bool(b) => *b,
			_ => panic!("value is not a bool"),
		}
	}

	pub fn as_method(&self) -> MethodPtr {
		match self {
			Self::Method(m) => m.clone(),
			_ => panic!("value is not a method"),
		}
	}

	pub fn new_native<T: 'static + NativeType>(obj: T) -> Value {
		Value::Native(NativeWrapper {
			obj: Rc::new(obj),
			descriptor: Box::new(NativeDescriptor::of::<T>()),
		})
	}
}

// FIXME: only used for unit tests
// switch to matching to remove this outright
impl PartialEq<Value> for Value {
	fn eq(&self, other: &Value) -> bool {
		match (self, &other) {
			(Value::Null(_), Value::Null(_)) => true,
			(Value::Int(a), Value::Int(b)) => a == b,
			(Value::Float(a), Value::Float(b)) => a == b,
			(Value::Bool(a), Value::Bool(b)) => a == b,
			(Value::String(a), Value::String(b)) => a == b,
			(Value::Type(a), Value::Type(b)) => a == b,
			(Value::Syntax(a), Value::Syntax(b)) => a == b,
			_ => false,
		}
	}
}
