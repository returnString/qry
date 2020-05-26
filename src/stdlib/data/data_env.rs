use super::{
	connect_sqlite, df_to_string, Connection, DataFrame, FilterStep, MutateStep, QueryPipeline,
	SelectStep,
};
use crate::runtime::{Builtin, Environment, EnvironmentPtr, Signature, Type, Value};
use crate::stdlib::ops::RUNTIME_OPS;
use std::rc::Rc;

pub fn env() -> EnvironmentPtr {
	let env = Environment::new("data");
	{
		let mut env = env.borrow_mut();
		let connection_type = &env.define_native_type::<Connection>();
		let pipeline_type = &env.define_native_type::<QueryPipeline>();
		let dataframe_type = &env.define_native_type::<DataFrame>();

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
				|_, args, _| {
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
				|_, args, _| {
					let conn = args[0].as_native::<Connection>();
					let table = args[1].as_string();
					Ok(Value::new_native(QueryPipeline::new(conn, table)))
				},
			),
		);

		env.update(
			"collect",
			Builtin::new_value(
				Signature::returning(dataframe_type).param("pipeline", pipeline_type),
				|_, args, _| {
					let pipeline = args[0].as_native::<QueryPipeline>();
					let batch = pipeline.collect()?;
					let df = DataFrame::new(vec![batch]);
					Ok(Value::new_native(df))
				},
			),
		);

		env.update(
			"render",
			Builtin::new_value(
				Signature::returning(&Type::String).param("pipeline", pipeline_type),
				|_, args, _| {
					let pipeline = args[0].as_native::<QueryPipeline>();
					let state = pipeline.generate()?;
					Ok(Value::String(state.query.into()))
				},
			),
		);

		env.update(
			"filter",
			Builtin::new_value(
				Signature::returning(pipeline_type)
					.param("pipeline", pipeline_type)
					.param("expr", &Type::SyntaxPlaceholder),
				|ctx, args, _| {
					let pipeline = args[0].as_native::<QueryPipeline>();
					let predicate = args[1].as_syntax();
					let step = FilterStep {
						ctx: ctx.clone(),
						predicate: predicate.clone(),
					};
					Ok(Value::new_native(pipeline.add(Rc::new(step))))
				},
			),
		);

		env.update(
			"select",
			Builtin::new_value(
				Signature::returning(pipeline_type)
					.param("pipeline", pipeline_type)
					.with_trailing(&Type::SyntaxPlaceholder),
				|ctx, args, _| {
					let pipeline = args[0].as_native::<QueryPipeline>();
					let cols = args[1..]
						.iter()
						.map(|a| a.as_syntax().clone())
						.collect::<Vec<_>>();

					let step = SelectStep {
						ctx: ctx.clone(),
						cols,
					};
					Ok(Value::new_native(pipeline.add(Rc::new(step))))
				},
			),
		);

		env.update(
			"mutate",
			Builtin::new_value(
				Signature::returning(pipeline_type)
					.param("pipeline", pipeline_type)
					.with_named_trailing(&Type::SyntaxPlaceholder),
				|ctx, args, named_args| {
					let pipeline = args[0].as_native::<QueryPipeline>();
					let new_cols = named_args
						.iter()
						.map(|(n, a)| (n.to_string(), a.as_syntax().clone()))
						.collect::<Vec<_>>();

					let step = MutateStep {
						ctx: ctx.clone(),
						new_cols,
					};
					Ok(Value::new_native(pipeline.add(Rc::new(step))))
				},
			),
		);

		env.update(
			"num_rows",
			Builtin::new_value(
				Signature::returning(&Type::Int).param("df", dataframe_type),
				|_, args, _| {
					let df = args[0].as_native::<DataFrame>();
					Ok(Value::Int(df.num_rows()))
				},
			),
		);

		env.update(
			"num_cols",
			Builtin::new_value(
				Signature::returning(&Type::Int).param("df", dataframe_type),
				|_, args, _| {
					let df = args[0].as_native::<DataFrame>();
					Ok(Value::Int(df.num_cols()))
				},
			),
		);

		env.update(
			"dimensions",
			Builtin::new_value(
				Signature::returning(&Type::List).param("df", dataframe_type),
				|_, args, _| {
					let df = args[0].as_native::<DataFrame>();
					Ok(Value::List(vec![
						Value::Int(df.num_rows()),
						Value::Int(df.num_cols()),
					]))
				},
			),
		);

		RUNTIME_OPS.with(|o| {
			let mut to_string = o.to_string.borrow_mut();
			to_string.register(Builtin::new(
				Signature::returning(&Type::String).param("obj", connection_type),
				|_, args, _| {
					Ok(Value::String(
						format!("Connection: {}", args[0].as_native::<Connection>().driver).into_boxed_str(),
					))
				},
			));

			to_string.register(Builtin::new(
				Signature::returning(&Type::String).param("obj", dataframe_type),
				|_, args, _| {
					let df = args[0].as_native::<DataFrame>();
					Ok(Value::String(df_to_string(&df).into_boxed_str()))
				},
			));

			to_string.register(Builtin::new(
				Signature::returning(&Type::String).param("obj", pipeline_type),
				|_, args, _| {
					let pipeline = args[0].as_native::<QueryPipeline>();
					Ok(Value::String(
						format!("QueryPipeline ({} steps)", pipeline.steps.len()).into_boxed_str(),
					))
				},
			));
		});
	}

	env
}
