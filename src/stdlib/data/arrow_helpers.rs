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

pub fn print_batch(batch: &RecordBatch) {
	let mut table = Table::new();

	let mut header = Vec::new();
	for field in batch.schema().fields() {
		header.push(Cell::new(field.name()));
	}

	table.set_titles(Row::new(header));

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
					_ => "unhandled type".to_string(),
				}
			};

			row.push(Cell::new(&val_str));
		}

		table.add_row(Row::new(row));
	}

	table.printstd();
}
