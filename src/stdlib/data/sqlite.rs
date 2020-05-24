use super::{Connection, ConnectionImpl, SqlError, SqlMetadata, SqlResult, SqlType};
use crate::runtime::{EvalResult, InterpreterError, Value};
use arrow::array::{ArrayBuilder, BooleanBuilder, Float64Builder, Int64Builder, StringBuilder};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use lazy_static::lazy_static;
use rusqlite::{Connection as SqliteConnection, Error as SqliteError, NO_PARAMS};
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
	static ref AFFINITY_MAP: HashMap<String, SqlType> = {
		let mut m = HashMap::new();
		m.insert("integer".into(), SqlType::Int);
		m.insert("text".into(), SqlType::String);
		m
	};
}

impl ConnectionImpl for SqliteConnectionImpl {
	fn get_table_metadata(&self, table: &str) -> SqlResult<SqlMetadata> {
		let names_query = format!("select * from {} limit 0", table);
		let stmt = self.conn.prepare(&names_query)?;

		let col_names = stmt.column_names();

		let typeof_calls = col_names
			.iter()
			.map(|n| format!("typeof({})", n))
			.collect::<Vec<_>>();

		let type_query = format!("select {} from {} limit 1", typeof_calls.join(", "), table);
		let mut stmt = self.conn.prepare(&type_query)?;
		let mut rows = stmt.query(NO_PARAMS)?;

		let mut metadata = SqlMetadata {
			col_types: HashMap::new(),
		};

		if let Some(row) = rows.next()? {
			for (col_idx, name) in col_names.iter().enumerate() {
				let col_affinity: String = row.get(col_idx)?;
				let col_type = AFFINITY_MAP[&col_affinity];
				metadata.col_types.insert(name.to_string(), col_type);
			}
		}

		Ok(metadata)
	}

	fn execute(&self, sql: &str) -> SqlResult<i64> {
		let rows = self.conn.execute(sql, NO_PARAMS)?;
		Ok(rows as i64)
	}

	fn collect(&self, query: &str, result_metadata: SqlMetadata) -> SqlResult<RecordBatch> {
		let mut stmt = self.conn.prepare(query)?;

		let col_metadata = stmt
			.columns()
			.iter()
			.map(|c| (c.name().to_string(), result_metadata.col_types[c.name()]))
			.collect::<Vec<_>>();

		let builders = col_metadata
			.iter()
			.map(|(_, c)| match c {
				SqlType::Int => box_builder(Int64Builder::new(0)),
				SqlType::Float => box_builder(Float64Builder::new(0)),
				SqlType::Bool => box_builder(BooleanBuilder::new(0)),
				SqlType::String => box_builder(StringBuilder::new(0)),
			})
			.collect::<Vec<_>>();

		let mut rows = stmt.query(NO_PARAMS)?;
		while let Some(row) = rows.next()? {
			for (col_idx, (builder, (_, col_type))) in builders.iter().zip(&col_metadata).enumerate() {
				match col_type {
					SqlType::Int => builder
						.borrow_mut()
						.as_any_mut()
						.downcast_mut::<Int64Builder>()
						.unwrap()
						.append_value(row.get(col_idx)?)?,
					SqlType::Float => builder
						.borrow_mut()
						.as_any_mut()
						.downcast_mut::<Float64Builder>()
						.unwrap()
						.append_value(row.get(col_idx)?)?,
					SqlType::Bool => builder
						.borrow_mut()
						.as_any_mut()
						.downcast_mut::<BooleanBuilder>()
						.unwrap()
						.append_value(row.get(col_idx)?)?,
					SqlType::String => builder
						.borrow_mut()
						.as_any_mut()
						.downcast_mut::<StringBuilder>()
						.unwrap()
						.append_value(&row.get::<usize, String>(col_idx)?)?,
				}
			}
		}

		let fields = col_metadata
			.iter()
			.map(|(n, c)| {
				Field::new(
					n,
					match c {
						SqlType::Int => DataType::Int64,
						SqlType::Float => DataType::Float64,
						SqlType::Bool => DataType::Boolean,
						SqlType::String => DataType::Utf8,
					},
					true,
				)
			})
			.collect();

		let cols = builders.iter().map(|b| b.borrow_mut().finish()).collect();
		Ok(RecordBatch::try_new(Arc::new(Schema::new(fields)), cols)?)
	}
}

impl From<SqliteError> for SqlError {
	fn from(err: SqliteError) -> Self {
		SqlError::UnknownError(format!("{}", err))
	}
}

impl From<SqlError> for InterpreterError {
	fn from(err: SqlError) -> Self {
		InterpreterError::UserCodeError(format!("{:?}", err))
	}
}

fn sqlite_connect_impl(connstring: &str) -> SqlResult<SqliteConnection> {
	Ok(SqliteConnection::open(connstring)?)
}

pub fn connect_sqlite(args: &[Value]) -> EvalResult {
	let connstring = args[0].as_string();
	let sqlite_conn = sqlite_connect_impl(connstring)?;

	Ok(Value::Native(Rc::new(Connection {
		driver: "sqlite".into(),
		conn_impl: Box::new(SqliteConnectionImpl { conn: sqlite_conn }),
	})))
}
