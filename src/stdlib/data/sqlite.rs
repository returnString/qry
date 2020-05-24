use super::{Connection, ConnectionImpl, SqlError, SqlResult, SqlTableMetadata, SqlType};
use crate::runtime::{EvalResult, InterpreterError, Value};
use arrow::array::{ArrayBuilder, Int64Builder};
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
	fn get_table_metadata(&self, table: &str) -> SqlResult<SqlTableMetadata> {
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

		let mut metadata = SqlTableMetadata {
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

	fn collect(&self, query: &str) -> SqlResult<RecordBatch> {
		let mut stmt = self.conn.prepare(query)?;

		let builders = stmt
			.columns()
			.iter()
			.map(|c| match c.decl_type() {
				// TODO: need to replace this with affinities
				Some(col_type) => match col_type {
					"integer" => (
						c.name().to_string(),
						SqlType::Int,
						box_builder(Int64Builder::new(0)),
					),
					_ => panic!("unhandled column type: {}", col_type),
				},
				None => panic!("column has no type: {}", c.name()),
			})
			.collect::<Vec<_>>();

		let mut rows = stmt.query(NO_PARAMS)?;
		while let Some(row) = rows.next()? {
			for (col_idx, (_, col_type, builder)) in builders.iter().enumerate() {
				match col_type {
					SqlType::Int => builder
						.borrow_mut()
						.as_any_mut()
						.downcast_mut::<Int64Builder>()
						.unwrap()
						.append_value(row.get(col_idx)?)?,
					_ => panic!("unhandled internal column type: {:?}", col_type),
				}
			}
		}

		let fields = builders
			.iter()
			.map(|(n, _, _)| Field::new(n, DataType::Int64, true))
			.collect();

		let cols = builders
			.iter()
			.map(|(_, _, b)| b.borrow_mut().finish())
			.collect();

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
