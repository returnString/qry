use crate::runtime::{
	Callable, Environment, EnvironmentPtr, RuntimeMethods, Signature, Type, Value,
};

pub fn env(methods: &RuntimeMethods) -> EnvironmentPtr {
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
			Type::Any,
		] {
			env.update(t.name(), Value::Type(t.clone()));
		}

		env.update("to_string", Value::Method(methods.to_string.clone()));

		env.define_builtin(
			"typeof",
			Signature::returning(&Type::Type).param("obj", &Type::Any),
			|_, args, _| Ok(Value::Type(args[0].runtime_type())),
		);

		env.define_builtin(
			"parse",
			Signature::returning(&Type::Syntax).param("code", &Type::SyntaxPlaceholder),
			|_, args, _| Ok(args[0].clone()),
		);

		env.define_builtin(
			"list",
			Signature::returning(&Type::List).with_trailing(&Type::Any),
			|_, args, _| Ok(Value::List(args.to_vec())),
		);

		env.define_builtin(
			"print",
			Signature::returning(&Type::Null).param("obj", &Type::Any),
			|ctx, args, _| {
				let str_val = ctx.methods.to_string.call(ctx, &[args[0].clone()], &[])?;
				println!("{}", str_val.as_string());
				Ok(Value::Null(()))
			},
		);
	}
	env
}
