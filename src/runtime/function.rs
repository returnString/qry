use super::{
	assign_value, eval, eval_multi, Callable, EnvironmentPtr, EvalContext, EvalError, EvalResult,
	Parameter, Signature, Value,
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
		.map(|p| match eval(ctx, &p.param_type)? {
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
			trailing_type: None,
			named_trailing_type: None,
			return_type: match eval(ctx, return_type)? {
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

	fn call(&self, ctx: &EvalContext, args: &[Value], _: &[(&str, Value)]) -> EvalResult {
		let func_body_env = self.env.borrow().child("funceval");

		for (value, param) in args.iter().zip(&self.signature.params) {
			func_body_env
				.borrow_mut()
				.update(&param.name, (*value).clone());
		}

		eval_multi(&ctx.child(func_body_env), &self.body)
	}
}
