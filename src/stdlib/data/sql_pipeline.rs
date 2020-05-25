use super::{expr_to_sql, Connection, SqlMetadata, SqlResult};
use crate::lang::Syntax;
use crate::runtime::{EvalContext, NativeType};
use arrow::record_batch::RecordBatch;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicI64, Ordering};

#[derive(Clone)]
pub struct QueryPipeline {
	conn: Rc<Connection>,
	pub steps: Vec<Rc<dyn PipelineStep>>,
}

impl NativeType for QueryPipeline {
	fn name() -> &'static str {
		"QueryPipeline"
	}
}

impl QueryPipeline {
	pub fn new(conn: Rc<Connection>, table: &str) -> Self {
		QueryPipeline {
			conn,
			steps: vec![Rc::new(FromStep {
				table: table.to_string(),
			})],
		}
	}

	pub fn add(&self, step: Rc<dyn PipelineStep>) -> QueryPipeline {
		let mut ret = self.clone();
		ret.steps.push(step);
		ret
	}

	pub fn generate(&self) -> SqlResult<RenderState> {
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

		Ok(state)
	}

	pub fn collect(&self) -> SqlResult<RecordBatch> {
		let state = self.generate()?;
		self.conn.conn_impl.collect(&state.query, state.metadata)
	}
}

#[derive(Clone)]
pub struct RenderState {
	pub conn: Rc<Connection>,
	pub metadata: SqlMetadata,
	counter: Rc<AtomicI64>,
	pub query: String,
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

pub trait PipelineStep {
	fn render(&self, state: RenderState) -> SqlResult<RenderState>;
}

#[derive(Clone)]
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

#[derive(Clone)]
pub struct FilterStep {
	pub ctx: EvalContext,
	pub predicate: Syntax,
}

impl PipelineStep for FilterStep {
	fn render(&self, state: RenderState) -> SqlResult<RenderState> {
		let predicate = expr_to_sql(&self.ctx, &self.predicate, &state.metadata)?;
		let column_names = state.metadata.col_types.keys().cloned().collect::<Vec<_>>();
		let select = column_names.join(", ");
		Ok(state.wrap(state.metadata.clone(), |sub| {
			format!("select {} from {} where {}", select, sub, predicate.text)
		}))
	}
}

#[derive(Clone)]
pub struct SelectStep {
	pub ctx: EvalContext,
	pub cols: Vec<Syntax>,
}

impl PipelineStep for SelectStep {
	fn render(&self, state: RenderState) -> SqlResult<RenderState> {
		let col_exprs = self
			.cols
			.iter()
			.map(|c| expr_to_sql(&self.ctx, c, &state.metadata))
			.collect::<SqlResult<Vec<_>>>()?;

		let col_names = col_exprs.iter().map(|e| e.text.clone()).collect::<Vec<_>>();

		let new_col_types = state
			.metadata
			.col_types
			.clone()
			.into_iter()
			.filter(|(k, _)| col_names.contains(k))
			.collect::<HashMap<_, _>>();

		let select = col_names.join(", ");
		Ok(state.wrap(
			SqlMetadata {
				col_types: new_col_types,
			},
			|sub| format!("select {} from {}", select, sub),
		))
	}
}
