use super::IntVector;
use crate::runtime::{NativeType, Value};
use arrow::array::{BooleanArray, Float64Array, Int64Array, StringArray};
use arrow::datatypes::DataType;
use arrow::record_batch::RecordBatch;
use prettytable::{Cell, Row, Table};

macro_rules! array_cast {
	($arrtype: ident, $arr: expr) => {
		$arr.as_any().downcast_ref::<$arrtype>().unwrap()
	};
}

macro_rules! array_val {
	($arrtype: ident, $arr: expr, $idx: expr, $value_func: ident) => {
		array_cast!($arrtype, $arr).$value_func($idx)
	};
	($arrtype: ident, $arr: expr, $idx: expr) => {
		array_val!($arrtype, $arr, $idx, value)
	};
}

pub fn df_to_string(df: &DataFrame) -> String {
	let batches = &df.batches;
	let mut table = Table::new();

	let mut header = Vec::new();
	for field in batches[0].schema().fields() {
		header.push(Cell::new(field.name()));
	}

	table.set_titles(Row::new(header));

	for batch in batches {
		for row_idx in 0..batch.num_rows() {
			let mut row = Vec::new();
			for col_idx in 0..batch.num_columns() {
				let col = batch.column(col_idx);
				let val_str = if col.is_null(row_idx) {
					"<NULL>".to_string()
				} else {
					match col.data_type() {
						DataType::Int64 => array_val!(Int64Array, col, row_idx).to_string(),
						DataType::Float64 => array_val!(Float64Array, col, row_idx).to_string(),
						DataType::Boolean => array_val!(BooleanArray, col, row_idx).to_string(),
						DataType::Utf8 => array_val!(StringArray, col, row_idx).to_string(),
						_ => unreachable!(),
					}
				};
				row.push(Cell::new(&val_str));
			}
			table.add_row(Row::new(row));
		}
	}

	table.to_string()
}

pub struct DataFrame {
	batches: Vec<RecordBatch>,
	num_rows: i64,
	num_cols: i64,
}

impl NativeType for DataFrame {
	fn name() -> &'static str {
		"DataFrame"
	}
}

impl DataFrame {
	pub fn new(batches: Vec<RecordBatch>) -> Self {
		let num_rows = batches.iter().map(|b| b.num_rows() as i64).sum();
		let num_cols = batches[0].num_columns() as i64;
		DataFrame {
			batches,
			num_rows,
			num_cols,
		}
	}

	pub fn num_rows(&self) -> i64 {
		self.num_rows
	}

	pub fn num_cols(&self) -> i64 {
		self.num_cols
	}

	pub fn col(&self, name: &str) -> Value {
		let (col_idx, field) = self.batches[0].schema().column_with_name(name).unwrap();
		let arrays = self
			.batches
			.iter()
			.map(|b| b.column(col_idx).clone())
			.collect::<Vec<_>>();

		match field.data_type() {
			DataType::Int64 => Value::new_native(IntVector::from_arrays(&arrays)),
			_ => panic!("unhandled datatype"),
		}
	}
}
