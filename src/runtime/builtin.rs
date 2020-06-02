use super::{Callable, EvalContext, EvalResult, Signature, Value};
use crate::lang::SourceLocation;
use std::fmt::Debug;
use std::rc::Rc;

type BuiltinFunc = fn(&EvalContext, &[Value], &[(&str, Value)]) -> EvalResult<Value>;

#[derive(Clone)]
pub struct Builtin {
	signature: Signature,
	func: BuiltinFunc,
}

impl Builtin {
	pub fn new(signature: Signature, func: BuiltinFunc) -> Rc<Builtin> {
		Rc::new(Builtin { signature, func })
	}

	pub fn new_value(signature: Signature, func: BuiltinFunc) -> Value {
		Value::Builtin(Self::new(signature, func))
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
		&SourceLocation::Native
	}

	fn name(&self) -> &str {
		"builtin" // TODO: ensure builtins track their names
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
