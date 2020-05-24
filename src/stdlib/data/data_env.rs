use super::{connect_sqlite, df_to_string, Connection, DataFrame, QueryPipeline};
use crate::runtime::{Builtin, Environment, EnvironmentPtr, Signature, Type, Value};
use crate::stdlib::ops::RUNTIME_OPS;
use std::any::TypeId;
use std::rc::Rc;

pub fn env() -> EnvironmentPtr {
	let connection_type = &Type::Native(TypeId::of::<Connection>());
	let pipeline_type = &Type::Native(TypeId::of::<QueryPipeline>());
	let dataframe_type = &Type::Native(TypeId::of::<DataFrame>());

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
			"table",
			Builtin::new_value(
				Signature::returning(pipeline_type)
					.param("connection", connection_type)
					.param("table", &Type::String),
				|args| {
					let conn = args[0].as_native::<Connection>();
					let table = args[1].as_string();
					Ok(Value::Native(Rc::new(QueryPipeline::new(conn, table))))
				},
			),
		);

		env.update(
			"collect",
			Builtin::new_value(
				Signature::returning(dataframe_type).param("pipeline", pipeline_type),
				|args| {
					let pipeline = args[0].as_native::<QueryPipeline>();
					let batch = pipeline.collect()?;
					let df = DataFrame::new(vec![batch]);
					Ok(Value::Native(Rc::new(df)))
				},
			),
		);

		env.update(
			"num_rows",
			Builtin::new_value(
				Signature::returning(&Type::Int).param("df", dataframe_type),
				|args| {
					let df = args[0].as_native::<DataFrame>();
					Ok(Value::Int(df.num_rows()))
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

		to_string.register(Builtin::new(
			Signature::returning(&Type::String).param("obj", dataframe_type),
			|args| {
				let df = args[0].as_native::<DataFrame>();
				Ok(Value::String(df_to_string(&df).into_boxed_str()))
			},
		));
	});

	env
}
