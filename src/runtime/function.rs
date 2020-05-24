use super::{
	assign_value, eval_in_env, eval_in_env_multi, Callable, EnvironmentPtr, EvalContext, EvalError,
	EvalResult, Parameter, Signature, Value,
};
use crate::lang::{ParameterDef, Syntax};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Function {
	pub body: Vec<Syntax>,
	pub signature: Signature,
	pub env: EnvironmentPtr,
}

pub fn eval_function(
	ctx: &EvalContext,
	name: &Option<String>,
	params: &[ParameterDef],
	return_type: &Syntax,
	body: &[Syntax],
) -> EvalResult {
	let param_types = params
		.iter()
		.map(|p| match eval_in_env(ctx, &p.param_type)? {
			Value::Type(t) => Ok(t),
			_ => Err(EvalError::NotType),
		})
		.collect::<Result<Vec<_>, _>>()?;

	let params = param_types
		.iter()
		.zip(params)
		.map(|(t, def)| Parameter {
			name: def.name.clone(),
			param_type: t.clone(),
		})
		.collect();

	let function = Value::Function(Rc::new(Function {
		body: body.to_vec(),
		signature: Signature {
			params,
			with_trailing: false,
			with_named_trailing: false,
			return_type: match eval_in_env(ctx, return_type)? {
				Value::Type(t) => t,
				_ => return Err(EvalError::NotType),
			},
		},
		env: ctx.env.clone(),
	}));

	if let Some(name) = name {
		assign_value(ctx, name, function.clone())?;
	}

	Ok(function)
}

impl Callable for Function {
	fn signature(&self) -> &Signature {
		&self.signature
	}

	fn call(
		&self,
		ctx: &EvalContext,
		args: &[(&String, &Value)],
		_: &[(&String, &Value)],
	) -> EvalResult {
		let func_body_env = self.env.borrow().child("funceval");

		for (name, value) in args {
			func_body_env.borrow_mut().update(name, (*value).clone());
		}

		eval_in_env_multi(&ctx.child(func_body_env), &self.body)
	}
}
