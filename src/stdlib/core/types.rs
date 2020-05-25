use crate::runtime::{Builtin, Environment, EnvironmentPtr, Signature, Type, Value};
use crate::stdlib::ops::RUNTIME_OPS;

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
				|_, args, _| Ok(Value::Type(args[0].runtime_type())),
			),
		);

		env.update(
			"parse",
			Builtin::new_value(
				Signature::returning(&Type::Syntax).param("code", &Type::SyntaxPlaceholder),
				|_, args, _| Ok(args[0].clone()),
			),
		);

		env.update(
			"list",
			Builtin::new_value(
				Signature::returning(&Type::List).with_trailing(&Type::Any),
				|_, args, _| Ok(Value::List(args.to_vec())),
			),
		)
	}
	env
}
