use super::{Connection, ConnectionImpl, SqlError, SqlResult};
use crate::runtime::{EvalResult, InterpreterError, Value};
use arrow::array::{ArrayBuilder, Int64Builder};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use rusqlite::{Connection as SqliteConnection, Error as SqliteError, NO_PARAMS};
use std::cell::RefCell;
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

#[derive(Clone, Copy)]
enum SqliteType {
	Int,
}

impl ConnectionImpl for SqliteConnectionImpl {
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
						SqliteType::Int,
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
					SqliteType::Int => builder
						.borrow_mut()
						.as_any_mut()
						.downcast_mut::<Int64Builder>()
						.unwrap()
						.append_value(row.get(col_idx)?)?,
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
