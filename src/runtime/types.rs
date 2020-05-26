use std::any::TypeId;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Type {
	Any,
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
	Native(Box<NativeDescriptor>),
	List,
}

impl Type {
	pub fn new_native<T: 'static + NativeType>() -> Type {
		Type::Native(Box::new(NativeDescriptor::of::<T>()))
	}

	pub fn name(&self) -> &str {
		match self {
			Self::Any => "Any",
			Self::Null => "Null",
			Self::Int => "Int",
			Self::Float => "Float",
			Self::Bool => "Bool",
			Self::String => "String",
			Self::Type => "Type",
			Self::Function => "Function",
			Self::Builtin => "Builtin",
			Self::Method => "Method",
			Self::Library => "Library",
			Self::Syntax => "Syntax",
			Self::SyntaxPlaceholder => "SyntaxPlaceholder",
			Self::Native(d) => d.name,
			Self::List => "List",
		}
	}
}

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
