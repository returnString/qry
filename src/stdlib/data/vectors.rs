use crate::lang::SourceLocation;
use crate::runtime::{EvalContext, EvalResult, NativeGenericType, NativeType, Type, Value};
use arrow::array::{ArrayRef, Int64Array, Int64Builder};
use std::sync::Arc;

pub struct Vector;

impl NativeType for Vector {
	fn name() -> &'static str {
		"Vector"
	}
}

impl NativeGenericType for Vector {
	fn resolve(ctx: &EvalContext, type_args: &[Type]) -> EvalResult<Type> {
		if type_args.len() != 1 {
			return Err(ctx.exception(&SourceLocation::Unknown, "expected one type argument"));
		}

		match type_args[0] {
			Type::Int => Ok(Type::new_native::<IntVector>()),
			_ => Err(ctx.exception(&SourceLocation::Unknown, "unsupported vector type")),
		}
	}
}

pub struct IntVector {
	data: Vec<ArrayRef>,
}

impl IntVector {
	pub fn from_arrays<'a, I>(arrays: I) -> Self
	where
		I: IntoIterator<Item = &'a ArrayRef>,
	{
		Self {
			data: arrays.into_iter().cloned().collect(),
		}
	}

	pub fn from_values(values: &[Value]) -> Self {
		let mut builder = Int64Builder::new(values.len());
		for value in values {
			builder.append_value(value.as_int()).unwrap();
		}

		Self {
			data: vec![Arc::new(builder.finish())],
		}
	}

	pub fn sum(&self) -> i64 {
		let mut ret = 0;
		for arr in &self.data {
			let concrete_arr = arr.as_any().downcast_ref::<Int64Array>().unwrap();
			for i in 0..concrete_arr.len() {
				ret += concrete_arr.value(i);
			}
		}

		ret
	}
}

impl NativeType for IntVector {
	fn name() -> &'static str {
		"IntVector"
	}
}
