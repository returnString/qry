use super::{eval_in_env, EvalContext, InterpreterError, Type, Value};
use crate::lang::Syntax;

#[derive(Debug, Clone)]
pub struct Parameter {
	pub name: String,
	pub param_type: Type,
}

#[derive(Debug, Clone)]
pub struct Signature {
	pub return_type: Type,
	pub params: Vec<Parameter>,
	pub with_trailing: bool,
	pub with_named_trailing: bool,
}

impl Signature {
	pub fn returning(return_type: Type) -> Self {
		Self {
			return_type,
			params: Vec::new(),
			with_trailing: false,
			with_named_trailing: false,
		}
	}

	pub fn param(&self, name: &str, param_type: Type) -> Self {
		let mut ret = self.clone();
		ret.params.push(Parameter {
			name: name.into(),
			param_type,
		});
		ret
	}
}

pub trait Callable {
	fn signature(&self) -> &Signature;
	fn call(
		&self,
		ctx: &EvalContext,
		args: &[(&String, Value)],
		named_varargs: &[(&String, Value)],
	) -> Result<Value, InterpreterError>;
}

pub fn eval_callable(
	ctx: &EvalContext,
	callable: &impl Callable,
	positional: &[Syntax],
) -> Result<Value, InterpreterError> {
	let sig = callable.signature();
	if positional.len() != sig.params.len() {
		return Err(InterpreterError::ArgMismatch);
	}

	let args = positional
		.iter()
		.zip(&sig.params)
		.map(|(a, p)| match p.param_type {
			Type::SyntaxPlaceholder => Ok(Value::Syntax(Box::new(a.clone()))),
			_ => eval_in_env(ctx, a),
		})
		.collect::<Result<Vec<_>, _>>()?;

	let combined = args
		.iter()
		.zip(&sig.params)
		.map(|(a, p)| (&p.name, a.clone()))
		.collect::<Vec<_>>();

	callable.call(ctx, combined.as_slice(), &[])
}
