use super::{expr_to_sql, Connection};
use crate::{EvalContext, EvalResult, NativeType, Type};
use arrow::record_batch::RecordBatch;
use indexmap::IndexMap;
use qry_lang::SyntaxNode;
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
	pub fn wrap(&self, metadata: &QueryMetadata, trailing: Option<&str>) -> RenderState {
		let col_strs = metadata
			.columns
			.iter()
			.map(|(n, c)| match &c.kind {
				ColumnKind::Named => n.clone(),
				ColumnKind::Computed(expr_text) => format!("{} as {}", expr_text, n),
			})
			.collect::<Vec<_>>();

		let new_cols = metadata
			.columns
			.iter()
			.map(|(n, c)| {
				(
					n.clone(),
					QueryColumn {
						kind: ColumnKind::Named,
						data_type: c.data_type.clone(),
					},
				)
			})
			.collect();

		let alias_index = self.counter.fetch_add(1, Ordering::SeqCst);
		let query = format!(
			"select {} from ({}) qry_{} {}",
			col_strs.join(", "),
			self.query,
			alias_index,
			trailing.unwrap_or(""),
		);

		RenderState {
			query,
			metadata: metadata.with_cols(new_cols),
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
		let columns = state
			.conn
			.conn_impl
			.get_relation_metadata(ctx, &self.table)?;
		let column_names = columns.keys().cloned().collect::<Vec<_>>();
		let select = column_names.join(", ");

		Ok(RenderState {
			metadata: QueryMetadata {
				columns,
				grouping: vec![],
			},
			query: format!("select {} from {}", select, self.table),
			..state
		})
	}
}

pub struct FilterStep {
	pub ctx: EvalContext,
	pub predicate: SyntaxNode,
}

impl PipelineStep for FilterStep {
	fn render(&self, _: &EvalContext, state: RenderState) -> EvalResult<RenderState> {
		let predicate = expr_to_sql(&self.ctx, &self.predicate, &state.metadata.columns)?;
		Ok(state.wrap(&state.metadata, Some(&format!("where {}", predicate.text))))
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
			.map(|c| expr_to_sql(&self.ctx, c, &state.metadata.columns))
			.collect::<EvalResult<Vec<_>>>()?;

		let col_names = col_exprs.iter().map(|e| e.text.clone()).collect::<Vec<_>>();

		let new_col_types = state
			.metadata
			.columns
			.clone()
			.into_iter()
			.filter(|(k, _)| col_names.contains(k))
			.collect::<ColumnMap>();

		Ok(state.wrap(&state.metadata.with_cols(new_col_types), None))
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
			.map(|(n, e)| Ok((n, expr_to_sql(&self.ctx, e, &state.metadata.columns)?)))
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

		Ok(state.wrap(&new_metadata, None))
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
				.map(|g| Ok(expr_to_sql(&self.ctx, g, &state.metadata.columns)?.text))
				.collect::<EvalResult<Vec<_>>>()?,
		);

		Ok(state.wrap(&new_metadata, None))
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
			.map(|(n, e)| Ok((n, expr_to_sql(&self.ctx, e, &state.metadata.columns)?)))
			.collect::<EvalResult<Vec<_>>>()?;

		let mut new_columns = ColumnMap::new();

		for name in &state.metadata.grouping {
			new_columns.insert(
				name.to_string(),
				QueryColumn {
					data_type: state.metadata.columns[name].data_type.clone(),
					kind: ColumnKind::Named,
				},
			);
		}

		for (name, sql_expr) in &aggregation_exprs {
			new_columns.insert(
				name.to_string(),
				QueryColumn {
					data_type: sql_expr.sql_type.clone(),
					kind: ColumnKind::Computed(sql_expr.text.clone()),
				},
			);
		}

		let new_metadata = QueryMetadata {
			columns: new_columns,
			grouping: vec![],
		};

		if state.metadata.grouping.is_empty() {
			Ok(state.wrap(&new_metadata, None))
		} else {
			Ok(state.wrap(
				&new_metadata,
				Some(&format!("group by {}", state.metadata.grouping.join(", "))),
			))
		}
	}
}
