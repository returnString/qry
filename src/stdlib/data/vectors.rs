use crate::lang::SourceLocation;
use crate::runtime::{EvalContext, EvalResult, NativeGenericType, NativeType, Type, Value};
use arrow::array::{Int64Array, Int64Builder};

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
	data: Int64Array,
}

impl IntVector {
	pub fn from_values(values: &[Value]) -> Self {
		let mut builder = Int64Builder::new(values.len());
		for value in values {
			builder.append_value(value.as_int()).unwrap();
		}

		Self {
			data: builder.finish(),
		}
	}

	pub fn sum(&self) -> i64 {
		let mut ret = 0;
		for i in 0..self.data.len() {
			ret += self.data.value(i);
		}
		ret
	}
}

impl NativeType for IntVector {
	fn name() -> &'static str {
		"IntVector"
	}
}
