use super::{Callable, EvalContext, EvalResult, Signature, Value};
use std::fmt::Debug;
use std::rc::Rc;

type BuiltinFunc = fn(&[Value]) -> EvalResult;

#[derive(Clone)]
pub struct Builtin {
	pub signature: Signature,
	pub func: BuiltinFunc,
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

	fn call(&self, _: &EvalContext, args: &[(&String, Value)], _: &[(&String, Value)]) -> EvalResult {
		let unnamed_args = args.iter().map(|(_, v)| v.clone()).collect::<Vec<_>>();
		(self.func)(&unnamed_args)
	}
}
