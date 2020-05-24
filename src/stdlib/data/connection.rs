use crate::runtime::{EvalError, Type};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatch;
use std::collections::HashMap;

pub type SqlResult<T> = Result<T, SqlError>;

#[derive(Debug)]
pub enum SqlError {
	ArrowError(ArrowError),
	UnknownError(String),
	SyntaxError,
	EvalError(EvalError),
}

impl From<SqlError> for EvalError {
	fn from(err: SqlError) -> Self {
		EvalError::UserCodeError(format!("{:?}", err))
	}
}

impl From<EvalError> for SqlError {
	fn from(err: EvalError) -> Self {
		SqlError::EvalError(err)
	}
}

#[derive(Debug, Clone)]
pub struct SqlMetadata {
	pub col_types: HashMap<String, Type>,
}

pub trait ConnectionImpl {
	fn get_table_metadata(&self, table: &str) -> SqlResult<SqlMetadata>;
	fn execute(&self, sql: &str) -> SqlResult<i64>;
	fn collect(&self, sql: &str, result_metadata: SqlMetadata) -> SqlResult<RecordBatch>;
}

pub struct Connection {
	pub driver: String,
	pub conn_impl: Box<dyn ConnectionImpl>,
}

impl Drop for Connection {
	fn drop(&mut self) {
		println!("connection is now dead")
	}
}

impl From<ArrowError> for SqlError {
	fn from(err: ArrowError) -> Self {
		SqlError::ArrowError(err)
	}
}
