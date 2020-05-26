use super::{eval, EvalContext, EvalError, EvalResult, Type, Value};
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
	fn call(&self, ctx: &EvalContext, args: &[Value], named_trailing: &[(&str, Value)])
		-> EvalResult;
}

fn typecheck_val(val: Value, expected_type: &Type) -> EvalResult {
	if expected_type == &Type::Any || expected_type == &val.runtime_type() {
		Ok(val)
	} else {
		Err(EvalError::TypeMismatch {
			expected: expected_type.clone(),
			actual: val.runtime_type(),
		})
	}
}

fn eval_arg(ctx: &EvalContext, param_type: &Type, expr: &Syntax) -> EvalResult {
	match param_type {
		Type::SyntaxPlaceholder => Ok(Value::Syntax(Box::new(expr.clone()))),
		_ => typecheck_val(eval(ctx, expr)?, param_type),
	}
}

pub fn eval_callable(
	ctx: &EvalContext,
	callable: &impl Callable,
	positional: &[Syntax],
	named_trailing: &[(&str, Syntax)],
) -> EvalResult {
	let sig = callable.signature();
	let num_supplied = positional.len();
	let num_expected_min = sig.params.len();
	if num_supplied < num_expected_min
		|| (sig.trailing_type.is_none() && num_supplied > num_expected_min)
	{
		return Err(EvalError::ArgMismatch);
	}

	if !named_trailing.is_empty() && sig.named_trailing_type.is_none() {
		return Err(EvalError::ArgMismatch);
	}

	let args = positional
		.iter()
		.enumerate()
		.map(|(i, s)| {
			let param_type = if i < num_expected_min {
				&sig.params[i].param_type
			} else {
				sig.trailing_type.as_ref().unwrap()
			};

			eval_arg(ctx, param_type, s)
		})
		.collect::<Result<Vec<_>, _>>()?;

	let named_args = named_trailing
		.iter()
		.map(|(n, s)| {
			let named_trailing_type = sig.named_trailing_type.as_ref().unwrap();
			let arg = eval_arg(ctx, &named_trailing_type, s)?;
			Ok((*n, arg))
		})
		.collect::<Result<Vec<_>, EvalError>>()?;

	let ret = callable.call(ctx, &args, &named_args)?;
	typecheck_val(ret, &sig.return_type)
}
