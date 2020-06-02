use crate::runtime::{EvalContext, EvalResult, NativeType, Type};
use arrow::record_batch::RecordBatch;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SqlMetadata {
	pub col_types: HashMap<String, Type>,
}

pub trait ConnectionImpl {
	fn get_table_metadata(&self, ctx: &EvalContext, table: &str) -> EvalResult<SqlMetadata>;
	fn execute(&self, ctx: &EvalContext, sql: &str) -> EvalResult<i64>;
	fn collect(
		&self,
		ctx: &EvalContext,
		sql: &str,
		result_metadata: SqlMetadata,
	) -> EvalResult<RecordBatch>;
}

pub struct Connection {
	pub driver: String,
	pub conn_impl: Box<dyn ConnectionImpl>,
}

impl NativeType for Connection {
	fn name() -> &'static str {
		"Connection"
	}
}
