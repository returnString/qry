use super::{Connection, SqlMetadata, SqlResult};
use arrow::record_batch::RecordBatch;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicI64, Ordering};

pub struct QueryPipeline {
	conn: Rc<Connection>,
	steps: Vec<Box<dyn PipelineStep>>,
}

impl QueryPipeline {
	pub fn new(conn: Rc<Connection>, table: &str) -> Self {
		QueryPipeline {
			conn,
			steps: vec![Box::new(FromStep {
				table: table.to_string(),
			})],
		}
	}

	pub fn collect(&self) -> SqlResult<RecordBatch> {
		let mut state = RenderState {
			conn: self.conn.clone(),
			metadata: SqlMetadata {
				col_types: HashMap::new(),
			},
			counter: Rc::new(AtomicI64::new(0)),
			query: "".into(),
		};

		for step in &self.steps {
			state = step.render(state)?;
		}

		self.conn.conn_impl.collect(&state.query, state.metadata)
	}
}

#[derive(Clone)]
pub struct RenderState {
	pub conn: Rc<Connection>,
	pub metadata: SqlMetadata,
	counter: Rc<AtomicI64>,
	query: String,
}

impl RenderState {
	pub fn subquery(&self) -> String {
		let alias_index = self.counter.fetch_add(1, Ordering::SeqCst);
		format!("({}) qry_{}", self.query, alias_index)
	}

	pub fn wrap<F>(&self, metadata: SqlMetadata, f: F) -> RenderState
	where
		F: Fn(&str) -> String,
	{
		let new_query = f(&self.subquery());
		RenderState {
			query: new_query,
			metadata,
			..self.clone()
		}
	}
}

trait PipelineStep {
	fn render(&self, state: RenderState) -> SqlResult<RenderState>;
}

pub struct FromStep {
	table: String,
}

impl PipelineStep for FromStep {
	fn render(&self, state: RenderState) -> SqlResult<RenderState> {
		let metadata = state.conn.conn_impl.get_table_metadata(&self.table)?;
		let column_names = metadata.col_types.keys().cloned().collect::<Vec<_>>();
		let select = column_names.join(", ");
		Ok(state.wrap(metadata, |_| {
			format!("select {} from {}", select, self.table)
		}))
	}
}
