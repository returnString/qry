use super::{connect_sqlite, Connection};
use crate::runtime::{Builtin, Environment, EnvironmentPtr, Signature, Type, Value};
use crate::stdlib::ops::RUNTIME_OPS;
use std::any::TypeId;

pub fn env() -> EnvironmentPtr {
	let connection_type = &Type::Native(TypeId::of::<Connection>());

	let env = Environment::new("data");
	{
		let mut env = env.borrow_mut();
		env.update(
			"connect_sqlite",
			Builtin::new_value(
				Signature::returning(connection_type).param("connstring", &Type::String),
				connect_sqlite,
			),
		);

		env.update(
			"execute",
			Builtin::new_value(
				Signature::returning(&Type::Int)
					.param("connection", connection_type)
					.param("query", &Type::String),
				|args| {
					let conn = args[0].as_native::<Connection>();
					let query = args[1].as_string();
					Ok(Value::Int(conn.conn_impl.execute(query)?))
				},
			),
		);

		env.update(
			"collect",
			Builtin::new_value(
				Signature::returning(&Type::Null).param("connection", connection_type),
				|args| {
					let conn = args[0].as_native::<Connection>();
					match conn.conn_impl.collect("select * from test_table") {
						Ok(_) => println!("got batch"),
						Err(err) => println!("execute failed: {:?}", err),
					};
					Ok(Value::Null(()))
				},
			),
		);
	}

	RUNTIME_OPS.with(|o| {
		let mut to_string = o.to_string.borrow_mut();
		to_string.register(Builtin::new(
			Signature::returning(&Type::String).param("obj", connection_type),
			|args| {
				Ok(Value::String(
					format!("Connection: {}", args[0].as_native::<Connection>().driver).into_boxed_str(),
				))
			},
		));
	});

	env
}
