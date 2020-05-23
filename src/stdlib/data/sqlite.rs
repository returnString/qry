use super::{Connection, ConnectionImpl, SqlError, SqlResult};
use crate::runtime::{EvalResult, InterpreterError, Value};
use arrow::datatypes::Schema;
use arrow::record_batch::RecordBatch;
use rusqlite::{Connection as SqliteConnection, Error as SqliteError, NO_PARAMS};
use std::rc::Rc;
use std::sync::Arc;

struct SqliteConnectionImpl {
	conn: SqliteConnection,
}

impl ConnectionImpl for SqliteConnectionImpl {
	fn execute(&self, sql: &str) -> SqlResult<i64> {
		let rows = self.conn.execute(sql, NO_PARAMS)?;
		Ok(rows as i64)
	}

	fn collect(&self, query: &str) -> SqlResult<RecordBatch> {
		let mut stmt = self.conn.prepare(query)?;
		let mut rows = stmt.query(NO_PARAMS)?;

		while let Some(row) = rows.next()? {}

		Ok(RecordBatch::try_new(Arc::new(Schema::empty()), vec![])?)
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
