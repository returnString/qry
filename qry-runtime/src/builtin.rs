use super::{Callable, EvalContext, EvalResult, Signature, Value};
use qry_lang::SourceLocation;
use std::fmt::Debug;
use std::rc::Rc;

pub type BuiltinFunc = fn(&EvalContext, &[Value], &[(&str, Value)]) -> EvalResult<Value>;

#[derive(Clone)]
pub struct Builtin {
	name: String,
	signature: Signature,
	func: BuiltinFunc,
	location: SourceLocation,
}

impl Builtin {
	pub fn new(
		name: &str,
		signature: Signature,
		location: SourceLocation,
		func: BuiltinFunc,
	) -> Rc<Builtin> {
		Rc::new(Builtin {
			name: name.into(),
			signature,
			func,
			location,
		})
	}
}

impl Debug for Builtin {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
		f.debug_struct("Builtin")
			.field("signature", &self.signature)
			.finish()
	}
}

impl Callable for Builtin {
	fn signature(&self) -> &Signature {
		&self.signature
	}

	fn source_location(&self) -> &SourceLocation {
		&self.location
	}

	fn name(&self) -> &str {
		&self.name
	}

	fn call(
		&self,
		ctx: &EvalContext,
		args: &[Value],
		named_trailing: &[(&str, Value)],
	) -> EvalResult<Value> {
		(self.func)(ctx, args, named_trailing)
	}
}
