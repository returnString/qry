use super::{Builtin, EnvironmentPtr, Function, Method, NativeDescriptor, NativeType, Type};
use crate::lang::Syntax;
use std::any::Any;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct NativeWrapper {
	descriptor: Box<NativeDescriptor>,
	obj: Rc<dyn Any>,
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
	Method(Rc<Method>),
	Library(EnvironmentPtr),
	Syntax(Box<Syntax>),
	Native(NativeWrapper),
	List(Vec<Value>),
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
			Self::List(_) => Type::List,
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

	pub fn as_int(&self) -> i64 {
		match self {
			Self::Int(i) => *i,
			_ => panic!("value is not an int"),
		}
	}

	pub fn as_method(&self) -> Rc<Method> {
		match self {
			Self::Method(m) => m.clone(),
			_ => panic!("value is not a method"),
		}
	}

	pub fn as_type(&self) -> Type {
		match self {
			Self::Type(t) => t.clone(),
			_ => panic!("value is not a type"),
		}
	}

	pub fn as_list(&self) -> &[Value] {
		match self {
			Self::List(l) => &l,
			_ => panic!("value is not a list"),
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
			(Value::List(a), Value::List(b)) => a == b,
			_ => false,
		}
	}
}
