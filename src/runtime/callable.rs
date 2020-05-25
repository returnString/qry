use super::{eval, EvalContext, EvalError, EvalResult, Type, Value};
use crate::lang::Syntax;
use std::iter::repeat;

#[derive(Debug, Clone)]
pub struct Parameter {
	pub name: String,
	pub param_type: Type,
}

#[derive(Debug, Clone)]
pub struct Signature {
	pub return_type: Type,
	pub params: Vec<Parameter>,
	pub trailing_type: Option<Type>,
	pub named_trailing_type: Option<Type>,
}

impl Signature {
	pub fn returning(return_type: &Type) -> Self {
		Self {
			return_type: return_type.clone(),
			params: Vec::new(),
			trailing_type: None,
			named_trailing_type: None,
		}
	}

	pub fn param(&self, name: &str, param_type: &Type) -> Self {
		let mut ret = self.clone();
		ret.params.push(Parameter {
			name: name.into(),
			param_type: param_type.clone(),
		});
		ret
	}

	pub fn with_trailing(&self, trailing_type: &Type) -> Self {
		let mut ret = self.clone();
		ret.trailing_type = Some(trailing_type.clone());
		ret
	}

	pub fn with_named_trailing(&self, named_trailing_type: &Type) -> Self {
		let mut ret = self.clone();
		ret.named_trailing_type = Some(named_trailing_type.clone());
		ret
	}
}

pub trait Callable {
	fn signature(&self) -> &Signature;
	fn call(
		&self,
		ctx: &EvalContext,
		args: &[(&String, &Value)],
		named_varargs: &[(&String, &Value)],
	) -> EvalResult;
}

pub fn eval_callable(
	ctx: &EvalContext,
	callable: &impl Callable,
	positional: &[Syntax],
) -> EvalResult {
	let sig = callable.signature();
	let num_supplied = positional.len();
	let num_expected_min = sig.params.len();
	if num_supplied < num_expected_min {
		return Err(EvalError::ArgMismatch);
	}

	let mut param_vec = sig.params.clone();
	if let Some(trailing_type) = &sig.trailing_type {
		let trailing_param = Parameter {
			name: "trailing".into(),
			param_type: trailing_type.clone(),
		};
		param_vec.extend(repeat(trailing_param).take(num_supplied - num_expected_min));
	} else if num_supplied > num_expected_min {
		return Err(EvalError::ArgMismatch);
	}

	let args = positional
		.iter()
		.zip(&param_vec)
		.map(|(a, p)| match p.param_type {
			Type::SyntaxPlaceholder => Ok(Value::Syntax(Box::new(a.clone()))),
			_ => eval(ctx, a),
		})
		.collect::<Result<Vec<_>, _>>()?;

	let combined = args
		.iter()
		.zip(&param_vec)
		.map(|(a, p)| (&p.name, a))
		.collect::<Vec<_>>();

	callable.call(ctx, combined.as_slice(), &[])
}
