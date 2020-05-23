use super::{Connection, ConnectionImpl, SqlError};
use crate::runtime::Value;
use arrow::datatypes::Schema;
use arrow::record_batch::RecordBatch;
use rusqlite::{Connection as SqliteConnection, Error as SqliteError, NO_PARAMS};
use std::rc::Rc;
use std::sync::Arc;

struct SqliteConnectionImpl {
	conn: SqliteConnection,
}

impl ConnectionImpl for SqliteConnectionImpl {
	fn collect(&self, query: &str) -> Result<RecordBatch, SqlError> {
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

pub fn connect_sqlite(_: &[Value]) -> Value {
	//let connstring = args[0].as_string();
	let connstring = ":memory:";
	let sqlite_conn = match SqliteConnection::open(connstring) {
		Ok(conn) => conn,
		Err(err) => panic!("failed to open sqlite db: {:?}", err),
	};

	Value::Native(Rc::new(Connection {
		driver: "sqlite".into(),
		conn_impl: Box::new(SqliteConnectionImpl { conn: sqlite_conn }),
	}))
}
