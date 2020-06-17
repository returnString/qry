use super::{expr_to_sql, Connection, SqlMetadata};
use crate::lang::SyntaxNode;
use crate::runtime::{EvalContext, EvalResult, NativeType};
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

	pub fn generate(&self, ctx: &EvalContext) -> EvalResult<RenderState> {
		let mut state = RenderState {
			conn: self.conn.clone(),
			metadata: QueryMetadata {
				grouping: vec![],
				cols: SqlMetadata {
					col_types: HashMap::new(),
				},
			},
			counter: Rc::new(AtomicI64::new(0)),
			query: "".into(),
		};

		for step in &self.steps {
			state = step.render(ctx, state)?;
		}

		Ok(state)
	}

	pub fn collect(&self, ctx: &EvalContext) -> EvalResult<RecordBatch> {
		let state = self.generate(ctx)?;
		self
			.conn
			.conn_impl
			.collect(ctx, &state.query, state.metadata.cols)
	}
}

#[derive(Clone)]
pub struct QueryMetadata {
	pub cols: SqlMetadata,
	pub grouping: Vec<String>,
}

impl QueryMetadata {
	pub fn with_cols(&self, cols: SqlMetadata) -> Self {
		QueryMetadata {
			cols,
			..self.clone()
		}
	}

	pub fn with_grouping(&self, grouping: Vec<String>) -> Self {
		QueryMetadata {
			grouping,
			..self.clone()
		}
	}
}

#[derive(Clone)]
pub struct RenderState {
	pub conn: Rc<Connection>,
	counter: Rc<AtomicI64>,
	pub query: String,
	pub metadata: QueryMetadata,
}

impl RenderState {
	pub fn subquery(&self) -> String {
		let alias_index = self.counter.fetch_add(1, Ordering::SeqCst);
		format!("({}) qry_{}", self.query, alias_index)
	}

	pub fn wrap<F>(&self, metadata: QueryMetadata, f: F) -> RenderState
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
	fn render(&self, ctx: &EvalContext, state: RenderState) -> EvalResult<RenderState>;
}

pub struct FromStep {
	table: String,
}

impl PipelineStep for FromStep {
	fn render(&self, ctx: &EvalContext, state: RenderState) -> EvalResult<RenderState> {
		let metadata = state.conn.conn_impl.get_table_metadata(ctx, &self.table)?;
		let column_names = metadata.col_types.keys().cloned().collect::<Vec<_>>();
		let select = column_names.join(", ");
		Ok(state.wrap(state.metadata.with_cols(metadata), |_| {
			format!("select {} from {}", select, self.table)
		}))
	}
}

pub struct FilterStep {
	pub ctx: EvalContext,
	pub predicate: SyntaxNode,
}

impl PipelineStep for FilterStep {
	fn render(&self, _: &EvalContext, state: RenderState) -> EvalResult<RenderState> {
		let predicate = expr_to_sql(&self.ctx, &self.predicate, &state.metadata.cols, false)?;
		let column_names = state
			.metadata
			.cols
			.col_types
			.keys()
			.cloned()
			.collect::<Vec<_>>();
		let select = column_names.join(", ");
		Ok(state.wrap(state.metadata.clone(), |sub| {
			format!("select {} from {} where {}", select, sub, predicate.text)
		}))
	}
}

pub struct SelectStep {
	pub ctx: EvalContext,
	pub cols: Vec<SyntaxNode>,
}

impl PipelineStep for SelectStep {
	fn render(&self, _: &EvalContext, state: RenderState) -> EvalResult<RenderState> {
		let col_exprs = self
			.cols
			.iter()
			.map(|c| expr_to_sql(&self.ctx, c, &state.metadata.cols, false))
			.collect::<EvalResult<Vec<_>>>()?;

		let col_names = col_exprs.iter().map(|e| e.text.clone()).collect::<Vec<_>>();

		let new_col_types = state
			.metadata
			.cols
			.col_types
			.clone()
			.into_iter()
			.filter(|(k, _)| col_names.contains(k))
			.collect::<HashMap<_, _>>();

		let select = col_names.join(", ");
		Ok(state.wrap(
			state.metadata.with_cols(SqlMetadata {
				col_types: new_col_types,
			}),
			|sub| format!("select {} from {}", select, sub),
		))
	}
}

pub struct MutateStep {
	pub ctx: EvalContext,
	pub new_cols: Vec<(String, SyntaxNode)>,
}

impl PipelineStep for MutateStep {
	fn render(&self, _: &EvalContext, state: RenderState) -> EvalResult<RenderState> {
		let new_col_exprs = self
			.new_cols
			.iter()
			.map(|(n, e)| Ok((n, expr_to_sql(&self.ctx, e, &state.metadata.cols, false)?)))
			.collect::<EvalResult<Vec<_>>>()?;

		let mut new_metadata = state.metadata.clone();
		for (name, sql_expr) in &new_col_exprs {
			new_metadata
				.cols
				.col_types
				.insert(name.to_string(), sql_expr.sql_type.clone());
		}

		let new_names = self.new_cols.iter().map(|(n, _)| n).collect::<Vec<_>>();

		let old_cols = state
			.metadata
			.cols
			.col_types
			.keys()
			.filter(|k| !new_names.contains(k))
			.cloned();

		let new_cols = new_col_exprs
			.iter()
			.map(|(n, c)| format!("{} as {}", c.text, n));

		let all_cols = old_cols.chain(new_cols).collect::<Vec<_>>();
		let select = all_cols.join(", ");

		Ok(state.wrap(new_metadata, |sub| {
			format!("select {} from {}", select, sub)
		}))
	}
}

pub struct GroupStep {
	pub ctx: EvalContext,
	pub grouping: Vec<SyntaxNode>,
}

impl PipelineStep for GroupStep {
	fn render(&self, _: &EvalContext, state: RenderState) -> EvalResult<RenderState> {
		let new_metadata = state.metadata.with_grouping(
			self
				.grouping
				.iter()
				.map(|g| Ok(expr_to_sql(&self.ctx, g, &state.metadata.cols, false)?.text))
				.collect::<EvalResult<Vec<_>>>()?,
		);

		Ok(state.wrap(new_metadata, |q| q.into()))
	}
}

pub struct AggregateStep {
	pub ctx: EvalContext,
	pub aggregations: Vec<(String, SyntaxNode)>,
}

impl PipelineStep for AggregateStep {
	fn render(&self, _: &EvalContext, state: RenderState) -> EvalResult<RenderState> {
		let aggregation_exprs = self
			.aggregations
			.iter()
			.map(|(n, e)| Ok((n, expr_to_sql(&self.ctx, e, &state.metadata.cols, true)?)))
			.collect::<EvalResult<Vec<_>>>()?;

		let mut new_metadata = state.metadata.clone();
		new_metadata.grouping.clear();

		for (name, sql_expr) in &aggregation_exprs {
			new_metadata
				.cols
				.col_types
				.insert(name.to_string(), sql_expr.sql_type.clone());
		}

		let aggregation_cols = aggregation_exprs
			.iter()
			.map(|(n, c)| format!("{} as {}", c.text, n));

		let all_cols = state
			.clone()
			.metadata
			.grouping
			.into_iter()
			.chain(aggregation_cols)
			.collect::<Vec<_>>();

		let select = all_cols.join(", ");
		let group_by = if state.metadata.grouping.is_empty() {
			"".into()
		} else {
			format!("group by {}", state.metadata.grouping.join(", "))
		};

		Ok(state.wrap(new_metadata, |sub| {
			format!("select {} from {} {}", select, sub, group_by)
		}))
	}
}
