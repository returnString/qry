use crate::runtime::{Builtin, Environment, EnvironmentPtr, Parameter, Signature, Type, Value};

fn typeof_func(args: &[Value]) -> Value {
	let target = &args[0];
	Value::Type(target.runtime_type())
}

fn parse_func(args: &[Value]) -> Value {
	args[0].clone()
}

pub fn types_module() -> EnvironmentPtr {
	let env = Environment::new("types");
	{
		let mut env = env.borrow_mut();
		env.update("Null", Value::Type(Type::Null));
		env.update("Int", Value::Type(Type::Int));
		env.update("Float", Value::Type(Type::Float));
		env.update("String", Value::Type(Type::String));
		env.update("Bool", Value::Type(Type::Bool));

		env.update(
			"typeof",
			Builtin::new_value(
				Signature {
					params: vec![Parameter {
						name: "obj".to_string(),
						param_type: Type::Null,
					}],
					return_type: Type::Type,
					with_trailing: false,
					with_named_trailing: false,
				},
				typeof_func,
			),
		);

		env.update(
			"parse",
			Builtin::new_value(
				Signature {
					params: vec![Parameter {
						name: "code".to_string(),
						param_type: Type::SyntaxPlaceholder,
					}],
					return_type: Type::Syntax,
					with_trailing: false,
					with_named_trailing: false,
				},
				parse_func,
			),
		);
	}
	env
}
