use crate::runtime::{
	Builtin, Environment, EnvironmentPtr, EvalContext, EvalResult, Signature, Type, Value,
};
use crate::stdlib::ops::RUNTIME_OPS;

fn typeof_func(_: &EvalContext, args: &[&Value]) -> EvalResult {
	let target = &args[0];
	Ok(Value::Type(target.runtime_type()))
}

fn parse_func(_: &EvalContext, args: &[&Value]) -> EvalResult {
	Ok(args[0].clone())
}

pub fn env() -> EnvironmentPtr {
	let env = Environment::new("core");
	{
		let mut env = env.borrow_mut();
		for t in &[
			Type::Null,
			Type::Int,
			Type::Float,
			Type::String,
			Type::Bool,
			Type::List,
		] {
			env.update(t.name(), Value::Type(t.clone()));
		}

		RUNTIME_OPS.with(|o| {
			env.update("to_string", Value::Method(o.to_string.clone()));
		});

		env.update(
			"typeof",
			Builtin::new_value(
				Signature::returning(&Type::Type).param("obj", &Type::Any),
				typeof_func,
			),
		);

		env.update(
			"parse",
			Builtin::new_value(
				Signature::returning(&Type::Syntax).param("code", &Type::SyntaxPlaceholder),
				parse_func,
			),
		);

		env.update(
			"list",
			Builtin::new_value(
				Signature::returning(&Type::List).with_trailing(&Type::Any),
				|_, args| Ok(Value::List(args.iter().cloned().cloned().collect())),
			),
		)
	}
	env
}
