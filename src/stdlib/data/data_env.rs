use super::{
	connect_sqlite, df_to_string, AggregateStep, Connection, DataFrame, FilterStep, GroupStep,
	IntVector, MutateStep, QueryPipeline, SelectStep, Vector,
};
use crate::runtime::{Environment, EnvironmentPtr, RuntimeMethods, Signature, Type, Value};
use std::rc::Rc;

pub fn env(methods: &RuntimeMethods) -> EnvironmentPtr {
	let env = Environment::new("data");
	{
		let mut env = env.borrow_mut();
		let connection_type = &env.define_native_type::<Connection>();
		let pipeline_type = &env.define_native_type::<QueryPipeline>();
		let dataframe_type = &env.define_native_type::<DataFrame>();
		env.define_native_generic_type::<Vector>();
		let intvector_type = &env.define_native_type::<IntVector>();

		let sum_method = env.define_method("sum", &["vec"], None, None);
		sum_method.register_builtin(
			Signature::returning(&Type::Int).param("vec", intvector_type),
			|_, args, _| {
				let vec = args[0].as_native::<IntVector>();
				Ok(Value::Int(vec.sum()))
			},
		);

		env.define_builtin(
			"intvec",
			Signature::returning(intvector_type).with_trailing(&Type::Int),
			|_, args, _| {
				let vec = IntVector::from_values(args);
				Ok(Value::new_native(vec))
			},
		);

		env.define_builtin(
			"connect_sqlite",
			Signature::returning(connection_type).param("connstring", &Type::String),
			connect_sqlite,
		);

		env.define_builtin(
			"execute",
			Signature::returning(&Type::Int)
				.param("connection", connection_type)
				.param("query", &Type::String),
			|ctx, args, _| {
				let conn = args[0].as_native::<Connection>();
				let query = args[1].as_string();
				Ok(Value::Int(conn.conn_impl.execute(ctx, query)?))
			},
		);

		env.define_builtin(
			"table",
			Signature::returning(pipeline_type)
				.param("connection", connection_type)
				.param("table", &Type::String),
			|_, args, _| {
				let conn = args[0].as_native::<Connection>();
				let table = args[1].as_string();
				Ok(Value::new_native(QueryPipeline::new(conn, table)))
			},
		);

		env.define_builtin(
			"collect",
			Signature::returning(dataframe_type).param("pipeline", pipeline_type),
			|ctx, args, _| {
				let pipeline = args[0].as_native::<QueryPipeline>();
				let batch = pipeline.collect(ctx)?;
				let df = DataFrame::new(vec![batch]);
				Ok(Value::new_native(df))
			},
		);

		env.define_builtin(
			"render",
			Signature::returning(&Type::String).param("pipeline", pipeline_type),
			|ctx, args, _| {
				let pipeline = args[0].as_native::<QueryPipeline>();
				let state = pipeline.generate(ctx)?;
				Ok(Value::String(state.query.into()))
			},
		);

		env.define_builtin(
			"filter",
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
		);

		env.define_builtin(
			"select",
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
		);

		env.define_builtin(
			"mutate",
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
		);

		env.define_builtin(
			"group_by",
			Signature::returning(pipeline_type)
				.param("pipeline", pipeline_type)
				.with_trailing(&Type::SyntaxPlaceholder),
			|ctx, args, _| {
				let pipeline = args[0].as_native::<QueryPipeline>();
				let grouping = args[1..]
					.iter()
					.map(|a| a.as_syntax().clone())
					.collect::<Vec<_>>();

				let step = GroupStep {
					ctx: ctx.clone(),
					grouping,
				};
				Ok(Value::new_native(pipeline.add(Rc::new(step))))
			},
		);

		env.define_builtin(
			"aggregate",
			Signature::returning(pipeline_type)
				.param("pipeline", pipeline_type)
				.with_named_trailing(&Type::SyntaxPlaceholder),
			|ctx, args, named_args| {
				let pipeline = args[0].as_native::<QueryPipeline>();
				let aggregations = named_args
					.iter()
					.map(|(n, a)| (n.to_string(), a.as_syntax().clone()))
					.collect::<Vec<_>>();

				let step = AggregateStep {
					ctx: ctx.clone(),
					aggregations,
				};
				Ok(Value::new_native(pipeline.add(Rc::new(step))))
			},
		);

		env.define_builtin(
			"num_rows",
			Signature::returning(&Type::Int).param("df", dataframe_type),
			|_, args, _| {
				let df = args[0].as_native::<DataFrame>();
				Ok(Value::Int(df.num_rows()))
			},
		);

		env.define_builtin(
			"num_cols",
			Signature::returning(&Type::Int).param("df", dataframe_type),
			|_, args, _| {
				let df = args[0].as_native::<DataFrame>();
				Ok(Value::Int(df.num_cols()))
			},
		);

		env.define_builtin(
			"dimensions",
			Signature::returning(&Type::List).param("df", dataframe_type),
			|_, args, _| {
				let df = args[0].as_native::<DataFrame>();
				Ok(Value::List(vec![
					Value::Int(df.num_rows()),
					Value::Int(df.num_cols()),
				]))
			},
		);

		methods.to_string.register_builtin(
			Signature::returning(&Type::String).param("obj", connection_type),
			|_, args, _| {
				Ok(Value::String(
					format!("Connection: {}", args[0].as_native::<Connection>().driver).into_boxed_str(),
				))
			},
		);

		methods.to_string.register_builtin(
			Signature::returning(&Type::String).param("obj", dataframe_type),
			|_, args, _| {
				let df = args[0].as_native::<DataFrame>();
				Ok(Value::String(df_to_string(&df).into_boxed_str()))
			},
		);

		methods.to_string.register_builtin(
			Signature::returning(&Type::String).param("obj", pipeline_type),
			|_, args, _| {
				let pipeline = args[0].as_native::<QueryPipeline>();
				Ok(Value::String(
					format!("QueryPipeline ({} steps)", pipeline.steps.len()).into_boxed_str(),
				))
			},
		);
	}

	env
}
