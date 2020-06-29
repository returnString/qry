use crate::{EvalContext, EvalResult, NativeGenericType, NativeType, Type, Value};
use arrow::array::{ArrayRef, Int64Array, Int64Builder};
use arrow::compute::{max, min, sum};
use qry_lang::SourceLocation;
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
			ret += sum(concrete_arr).unwrap_or(0);
		}

		ret
	}

	pub fn min(&self) -> Option<i64> {
		let mut chunk_mins = Vec::new();
		for arr in &self.data {
			let concrete_arr = arr.as_any().downcast_ref::<Int64Array>().unwrap();

			if let Some(val) = min(concrete_arr) {
				chunk_mins.push(val);
			}
		}

		chunk_mins.into_iter().min()
	}

	pub fn max(&self) -> Option<i64> {
		let mut chunk_maxes = Vec::new();
		for arr in &self.data {
			let concrete_arr = arr.as_any().downcast_ref::<Int64Array>().unwrap();

			if let Some(val) = max(concrete_arr) {
				chunk_maxes.push(val);
			}
		}

		chunk_maxes.into_iter().max()
	}
}

impl NativeType for IntVector {
	fn name() -> &'static str {
		"IntVector"
	}
}
