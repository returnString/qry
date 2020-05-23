use crate::runtime::{Builtin, Environment, EnvironmentPtr, Parameter, Signature, Type, Value};
use crate::stdlib::ops::RUNTIME_OPS;
use std::any::TypeId;
use std::rc::Rc;

mod connection;

fn connect_func(_: &[Value]) -> Value {
	Value::Native(Rc::new(connection::Connection {
		driver: "sqlite".into(),
	}))
}

fn use_conn(args: &[Value]) -> Value {
	let conn = args[0].as_native::<connection::Connection>();
	println!("{:?}", conn);
	Value::Native(conn)
}

pub fn env() -> EnvironmentPtr {
	let connection_type = Type::Native(TypeId::of::<connection::Connection>());

	let env = Environment::new("data");
	{
		let mut env = env.borrow_mut();
		env.update(
			"connect",
			Builtin::new_value(
				Signature {
					params: vec![],
					return_type: connection_type.clone(),
					with_trailing: false,
					with_named_trailing: false,
				},
				connect_func,
			),
		);
		env.update(
			"use_conn",
			Builtin::new_value(
				Signature {
					params: vec![Parameter {
						name: "conn".to_string(),
						param_type: connection_type.clone(),
					}],
					return_type: Type::Null,
					with_trailing: false,
					with_named_trailing: false,
				},
				use_conn,
			),
		);
	}

	RUNTIME_OPS.with(|o| {
		let mut to_string = o.to_string.borrow_mut();
		to_string.register(Builtin::new(
			Signature::returning(Type::String).param("obj", connection_type),
			|args| {
				Value::String(
					format!(
						"Connection: {}",
						args[0].as_native::<connection::Connection>().driver
					)
					.into_boxed_str(),
				)
			},
		));
	});

	env
}
