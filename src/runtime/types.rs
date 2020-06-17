use super::{EvalContext, EvalResult};
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

	pub fn new_native_generic<T: 'static + NativeGenericType>() -> Type {
		Type::Native(Box::new(NativeDescriptor::of_generic::<T>()))
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

type GenericResolver = fn(&EvalContext, &[Type]) -> EvalResult<Type>;

#[derive(Clone)]
pub struct NativeDescriptor {
	id: TypeId,
	name: &'static str,
	pub generic_lookup: Option<GenericResolver>,
}

impl std::hash::Hash for NativeDescriptor {
	fn hash<H>(&self, h: &mut H)
	where
		H: std::hash::Hasher,
	{
		self.id.hash(h)
	}
}

impl std::cmp::PartialEq for NativeDescriptor {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}
}

impl std::cmp::Eq for NativeDescriptor {}

impl std::fmt::Debug for NativeDescriptor {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
		f.debug_struct("NativeDescriptor")
			.field("id", &self.id)
			.field("name", &self.name)
			.finish()
	}
}

impl NativeDescriptor {
	pub fn of<T: 'static + NativeType>() -> NativeDescriptor {
		NativeDescriptor {
			id: TypeId::of::<T>(),
			name: T::name(),
			generic_lookup: None,
		}
	}

	pub fn of_generic<T: 'static + NativeGenericType>() -> NativeDescriptor {
		NativeDescriptor {
			id: TypeId::of::<T>(),
			name: T::name(),
			generic_lookup: Some(T::resolve),
		}
	}
}

pub trait NativeType {
	fn name() -> &'static str;
}

pub trait NativeGenericType: NativeType {
	fn resolve(ctx: &EvalContext, type_args: &[Type]) -> EvalResult<Type>;
}
