use super::{expr_to_sql, Connection};
use crate::lang::SyntaxNode;
use crate::runtime::{EvalContext, EvalResult, NativeType, Type};
use arrow::record_batch::RecordBatch;
use indexmap::IndexMap;
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
				columns: IndexMap::new(),
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
			.collect(ctx, &state.query, &state.metadata.columns)
	}
}

#[derive(Clone)]
pub enum ColumnKind {
	Named,
	Computed(String),
}

#[derive(Clone)]
pub struct QueryColumn {
	pub data_type: Type,
	pub kind: ColumnKind,
}

pub type ColumnMap = IndexMap<String, QueryColumn>;

#[derive(Clone)]
pub struct QueryMetadata {
	pub columns: ColumnMap,
	pub grouping: Vec<String>,
}

impl QueryMetadata {
	pub fn with_cols(&self, columns: ColumnMap) -> Self {
		QueryMetadata {
			columns,
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
		let metadata = state
			.conn
			.conn_impl
			.get_relation_metadata(ctx, &self.table)?;
		let column_names = metadata.keys().cloned().collect::<Vec<_>>();
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
		let predicate = expr_to_sql(&self.ctx, &self.predicate, &state.metadata.columns, false)?;
		let column_names = state.metadata.columns.keys().cloned().collect::<Vec<_>>();
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
			.map(|c| expr_to_sql(&self.ctx, c, &state.metadata.columns, false))
			.collect::<EvalResult<Vec<_>>>()?;

		let col_names = col_exprs.iter().map(|e| e.text.clone()).collect::<Vec<_>>();

		let new_col_types = state
			.metadata
			.columns
			.clone()
			.into_iter()
			.filter(|(k, _)| col_names.contains(k))
			.collect::<ColumnMap>();

		let select = col_names.join(", ");
		Ok(state.wrap(state.metadata.with_cols(new_col_types), |sub| {
			format!("select {} from {}", select, sub)
		}))
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
			.map(|(n, e)| {
				Ok((
					n,
					expr_to_sql(&self.ctx, e, &state.metadata.columns, false)?,
				))
			})
			.collect::<EvalResult<Vec<_>>>()?;

		let mut new_metadata = state.metadata.clone();
		for (name, sql_expr) in &new_col_exprs {
			new_metadata.columns.insert(
				name.to_string(),
				QueryColumn {
					data_type: sql_expr.sql_type.clone(),
					kind: ColumnKind::Computed(sql_expr.text.clone()),
				},
			);
		}

		let new_names = self.new_cols.iter().map(|(n, _)| n).collect::<Vec<_>>();

		let old_cols = state
			.metadata
			.columns
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
				.map(|g| Ok(expr_to_sql(&self.ctx, g, &state.metadata.columns, false)?.text))
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
			.map(|(n, e)| Ok((n, expr_to_sql(&self.ctx, e, &state.metadata.columns, true)?)))
			.collect::<EvalResult<Vec<_>>>()?;

		let mut new_metadata = state.metadata.clone();
		new_metadata.grouping.clear();

		for (name, sql_expr) in &aggregation_exprs {
			new_metadata.columns.insert(
				name.to_string(),
				QueryColumn {
					data_type: sql_expr.sql_type.clone(),
					kind: ColumnKind::Computed(sql_expr.text.clone()),
				},
			);
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
