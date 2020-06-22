use super::ColumnMap;
use crate::{EvalContext, EvalResult, NativeType};
use arrow::record_batch::RecordBatch;

pub trait ConnectionImpl {
	fn get_relation_metadata(&self, ctx: &EvalContext, table: &str) -> EvalResult<ColumnMap>;
	fn execute(&self, ctx: &EvalContext, sql: &str) -> EvalResult<i64>;
	fn collect(
		&self,
		ctx: &EvalContext,
		sql: &str,
		result_metadata: &ColumnMap,
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
