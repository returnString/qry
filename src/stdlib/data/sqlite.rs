use super::{Connection, ConnectionImpl, SqlMetadata};
use crate::lang::SourceLocation;
use crate::runtime::{EvalContext, EvalResult, Type, Value};
use arrow::array::{ArrayBuilder, BooleanBuilder, Float64Builder, Int64Builder, StringBuilder};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::error::Result as ArrowResult;
use arrow::record_batch::RecordBatch;
use lazy_static::lazy_static;
use rusqlite::types::ValueRef;
use rusqlite::{Connection as SqliteConnection, Result as SqliteResult, NO_PARAMS};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

struct SqliteConnectionImpl {
	conn: SqliteConnection,
}

fn box_builder<T>(builder: T) -> Rc<RefCell<dyn ArrayBuilder>>
where
	T: ArrayBuilder,
{
	Rc::new(RefCell::new(builder))
}

lazy_static! {
	static ref AFFINITY_MAP: HashMap<String, Type> = {
		let mut m = HashMap::new();
		m.insert("integer".into(), Type::Int);
		m.insert("text".into(), Type::String);
		m
	};
}

pub fn arrow_op<T>(ctx: &EvalContext, res: ArrowResult<T>) -> EvalResult<T> {
	match res {
		Ok(val) => Ok(val),
		Err(err) => Err(ctx.exception(&SourceLocation::Native, format!("arrow error: {}", err))),
	}
}

pub fn sqlite_op<T>(ctx: &EvalContext, res: SqliteResult<T>) -> EvalResult<T> {
	match res {
		Ok(val) => Ok(val),
		Err(err) => Err(ctx.exception(&SourceLocation::Native, format!("sqlite error: {}", err))),
	}
}

macro_rules! write_cell {
	($ctx: expr, $builder: expr, $builder_type: ty, $row: expr, $col_idx: expr, $write_func: expr, $getter: expr) => {{
		let mut builder = $builder.borrow_mut();
		let concrete_builder = builder
			.as_any_mut()
			.downcast_mut::<$builder_type>()
			.unwrap();

		if let ValueRef::Null = $row.get_raw($col_idx) {
			arrow_op($ctx, concrete_builder.append_null())?;
		} else {
			let val = sqlite_op($ctx, $getter())?;
			arrow_op($ctx, $write_func(concrete_builder, &val))?
		}
		}};
	($ctx: expr, $builder: expr, $builder_type: ty, $row: expr, $col_idx: expr) => {{
		write_cell!(
			$ctx,
			$builder,
			$builder_type,
			$row,
			$col_idx,
			|b: &mut $builder_type, &val| b.append_value(val),
			|| $row.get($col_idx)
		)
		}};
}

impl ConnectionImpl for SqliteConnectionImpl {
	fn get_table_metadata(&self, ctx: &EvalContext, table: &str) -> EvalResult<SqlMetadata> {
		let names_query = format!("select * from {} limit 0", table);
		let stmt = sqlite_op(ctx, self.conn.prepare(&names_query))?;

		let col_names = stmt.column_names();

		let typeof_calls = col_names
			.iter()
			.map(|n| format!("typeof({})", n))
			.collect::<Vec<_>>();

		let type_query = format!("select {} from {} limit 1", typeof_calls.join(", "), table);
		let mut stmt = sqlite_op(ctx, self.conn.prepare(&type_query))?;
		let mut rows = sqlite_op(ctx, stmt.query(NO_PARAMS))?;

		let mut metadata = SqlMetadata {
			col_types: HashMap::new(),
		};

		if let Some(row) = sqlite_op(ctx, rows.next())? {
			for (col_idx, name) in col_names.iter().enumerate() {
				let col_affinity: String = sqlite_op(ctx, row.get(col_idx))?;
				let col_type = AFFINITY_MAP[&col_affinity].clone();
				metadata.col_types.insert(name.to_string(), col_type);
			}
		}

		Ok(metadata)
	}

	fn execute(&self, ctx: &EvalContext, sql: &str) -> EvalResult<i64> {
		let rows = sqlite_op(ctx, self.conn.execute(sql, NO_PARAMS))?;
		Ok(rows as i64)
	}

	fn collect(
		&self,
		ctx: &EvalContext,
		query: &str,
		result_metadata: SqlMetadata,
	) -> EvalResult<RecordBatch> {
		let mut stmt = sqlite_op(ctx, self.conn.prepare(query))?;

		let col_metadata = stmt
			.columns()
			.iter()
			.map(|c| {
				(
					c.name().to_string(),
					result_metadata.col_types[c.name()].clone(),
				)
			})
			.collect::<Vec<_>>();

		let builders = col_metadata
			.iter()
			.map(|(_, c)| match c {
				Type::Int => box_builder(Int64Builder::new(0)),
				Type::Float => box_builder(Float64Builder::new(0)),
				Type::Bool => box_builder(BooleanBuilder::new(0)),
				Type::String => box_builder(StringBuilder::new(0)),
				_ => unimplemented!(),
			})
			.collect::<Vec<_>>();

		let mut rows = sqlite_op(ctx, stmt.query(NO_PARAMS))?;
		while let Some(row) = sqlite_op(ctx, rows.next())? {
			for (col_idx, (builder, (_, col_type))) in builders.iter().zip(&col_metadata).enumerate() {
				match col_type {
					Type::Int => write_cell!(ctx, builder, Int64Builder, row, col_idx),
					Type::Float => write_cell!(ctx, builder, Float64Builder, row, col_idx),
					Type::Bool => write_cell!(ctx, builder, BooleanBuilder, row, col_idx),
					Type::String => write_cell!(
						ctx,
						builder,
						StringBuilder,
						row,
						col_idx,
						|b: &mut StringBuilder, s| b.append_value(s),
						|| row.get::<usize, String>(col_idx)
					),
					_ => unimplemented!(),
				};
			}
		}

		let fields = col_metadata
			.iter()
			.map(|(n, c)| {
				Field::new(
					n,
					match c {
						Type::Int => DataType::Int64,
						Type::Float => DataType::Float64,
						Type::Bool => DataType::Boolean,
						Type::String => DataType::Utf8,
						_ => unimplemented!(),
					},
					true,
				)
			})
			.collect();

		let cols = builders.iter().map(|b| b.borrow_mut().finish()).collect();
		Ok(arrow_op(
			ctx,
			RecordBatch::try_new(Arc::new(Schema::new(fields)), cols),
		)?)
	}
}

pub fn connect_sqlite(ctx: &EvalContext, args: &[Value], _: &[(&str, Value)]) -> EvalResult<Value> {
	let connstring = args[0].as_string();
	let sqlite_conn = match SqliteConnection::open(connstring) {
		Ok(conn) => conn,
		Err(err) => {
			return Err(ctx.exception(&SourceLocation::Native, format!("sqlite error: {}", err)))
		}
	};

	Ok(Value::new_native(Connection {
		driver: "sqlite".into(),
		conn_impl: Box::new(SqliteConnectionImpl { conn: sqlite_conn }),
	}))
}
