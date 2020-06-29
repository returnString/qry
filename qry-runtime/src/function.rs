use super::{
	assign_value, eval, eval_multi, Callable, Environment, EvalContext, EvalResult, Parameter,
	Signature, Value,
};
use qry_lang::{FunctionHeader, ParameterDef, SourceLocation, SyntaxNode};
use std::rc::Rc;

#[derive(Debug)]
pub struct Function {
	pub body: Vec<SyntaxNode>,
	pub signature: Signature,
	pub env: Environment,
	pub name: String,
	pub location: SourceLocation,
}

pub fn eval_function_decl(
	ctx: &EvalContext,
	location: &SourceLocation,
	header: &FunctionHeader<SyntaxNode>,
	params: &[ParameterDef<SyntaxNode>],
	return_type: &SyntaxNode,
	body: &[SyntaxNode],
) -> EvalResult<Value> {
	let param_types = params
		.iter()
		.map(|p| match eval(ctx, &p.param_type)? {
			Value::Type(t) => Ok(t),
			_ => Err(ctx.exception(&p.param_type.location, "expected a type")),
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

	let function = Rc::new(Function {
		body: body.to_vec(),
		signature: Signature {
			params,
			trailing_type: None,
			named_trailing_type: None,
			return_type: match eval(ctx, return_type)? {
				Value::Type(t) => t,
				_ => return Err(ctx.exception(&return_type.location, "expected a type")),
			},
		},
		env: (*ctx.env).clone(),
		name: match header {
			FunctionHeader::Function(Some(name)) => name,
			FunctionHeader::Function(None) => "<anonymous function>",
			FunctionHeader::MethodImpl { .. } => "<method impl>",
		}
		.into(),
		location: location.clone(),
	});

	let function_val = Value::Function(function.clone());

	match header {
		FunctionHeader::Function(Some(name)) => {
			assign_value(ctx, name, function_val.clone())?;
		}
		FunctionHeader::MethodImpl { impl_for } => {
			let method = eval(ctx, impl_for)?.as_method();
			method.register(function);
		}
		_ => (),
	};

	Ok(function_val)
}

impl Callable for Function {
	fn signature(&self) -> &Signature {
		&self.signature
	}

	fn source_location(&self) -> &SourceLocation {
		&self.location
	}

	fn name(&self) -> &str {
		&self.name
	}

	fn call(&self, ctx: &EvalContext, args: &[Value], _: &[(&str, Value)]) -> EvalResult<Value> {
		let func_body_env = self.env.child("funceval");

		for (value, param) in args.iter().zip(&self.signature.params) {
			func_body_env.update(&param.name, (*value).clone());
		}

		eval_multi(&ctx.child(func_body_env), &self.body)
	}
}
