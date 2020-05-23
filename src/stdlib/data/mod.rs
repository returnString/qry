use crate::runtime::{Builtin, Environment, EnvironmentPtr, Parameter, Signature, Type, Value};
use std::any::TypeId;
use std::rc::Rc;

mod connection;

fn connect_func(_: &[Value]) -> Value {
	Value::Native(Rc::new(connection::Connection {}))
}

fn use_conn(args: &[Value]) -> Value {
	let conn = args[0].as_native::<connection::Connection>();
	println!("{:?}", conn);
	Value::Native(conn)
}

pub fn data_module() -> EnvironmentPtr {
	let env = Environment::new("data");
	{
		let mut env = env.borrow_mut();
		env.update(
			"connect",
			Builtin::new_value(
				Signature {
					params: vec![],
					return_type: Type::Native(TypeId::of::<connection::Connection>()),
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
						param_type: Type::Native(TypeId::of::<connection::Connection>()),
					}],
					return_type: Type::Null,
					with_trailing: false,
					with_named_trailing: false,
				},
				use_conn,
			),
		);
	}
	env
}
