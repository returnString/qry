use arrow::error::ArrowError;
use arrow::record_batch::RecordBatch;

pub type SqlResult<T> = Result<T, SqlError>;

#[derive(Debug)]
pub enum SqlError {
	ArrowError(ArrowError),
	UnknownError(String),
}

pub trait ConnectionImpl {
	fn execute(&self, sql: &str) -> SqlResult<i64>;
	fn collect(&self, sql: &str) -> SqlResult<RecordBatch>;
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
