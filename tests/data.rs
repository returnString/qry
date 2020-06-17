use qry::runtime::Value;

pub mod helpers;

const TABLE_BOOTSTRAP: &str = r#"
use data::*
conn <- connect_sqlite(":memory:")
execute(conn, "create table test_table (name varchar(255), age integer)")
execute(conn, "insert into test_table (name, age) values ('ruan', 26), ('ruanlater', 27), ('thirdperson', 27), ('ancient one', null)")
test_table <- table(conn, "test_table")
"#;

fn with_table_bootstrap(query: &str) -> String {
	format!("{}\n{}", TABLE_BOOTSTRAP, query)
}

#[test]
fn test_data_sqlite() {
	helpers::eval_expect_values(&[
		(
			&with_table_bootstrap(r#"test_table |> collect() |> num_rows()"#),
			Value::Int(4),
		),
		(
			&with_table_bootstrap(r#"test_table |> filter(name == "ruan") |> collect() |> num_rows()"#),
			Value::Int(1),
		),
		(
			&with_table_bootstrap(
				r#"name_to_find <- "ancient one"
				test_table |> filter(name == {{name_to_find}}) |> collect() |> num_rows()"#,
			),
			Value::Int(1),
		),
		(
			&with_table_bootstrap(r#"test_table |> filter(age == 27) |> collect() |> num_rows()"#),
			Value::Int(2),
		),
		(
			&with_table_bootstrap(
				r#"test_table |> filter(age == 27 | age == 26) |> collect() |> num_rows()"#,
			),
			Value::Int(3),
		),
		(
			&with_table_bootstrap(
				r#"test_table |> filter(age == 27 & name == "ruanlater") |> collect() |> num_rows()"#,
			),
			Value::Int(1),
		),
		(
			&with_table_bootstrap(
				r#"test_table |>
					filter(switch age {
						26 => true
						27 => false
					})
					|> collect() |> num_rows()"#,
			),
			Value::Int(1),
		),
		(
			&with_table_bootstrap(r#"test_table |> select(age) |> collect() |> num_cols()"#),
			Value::Int(1),
		),
		(
			&with_table_bootstrap(r#"test_table |> select(age, name) |> collect() |> num_cols()"#),
			Value::Int(2),
		),
		(
			&with_table_bootstrap(r#"test_table |> collect() |> num_cols()"#),
			Value::Int(2),
		),
		(
			&with_table_bootstrap(
				r#"
				test_table
					|> mutate(new_col = age * 2)
					|> filter(new_col == 52)
					|> collect()
					|> dimensions()
				"#,
			),
			Value::List(vec![Value::Int(1), Value::Int(3)]),
		),
		(
			&with_table_bootstrap(
				r#"
				test_table
					|> mutate(age = age - 1)
					|> filter(age == 26)
					|> collect()
					|> dimensions()
				"#,
			),
			Value::List(vec![Value::Int(2), Value::Int(2)]),
		),
		(
			&with_table_bootstrap(
				r#"
				test_table
					|> group_by(age)
					|> aggregate(total_age = sum(age))
					|> collect()
					|> dimensions()
				"#,
			),
			Value::List(vec![Value::Int(3), Value::Int(2)]),
		),
		(
			&with_table_bootstrap(
				r#"
				test_table
					|> aggregate(total_age = sum(age))
					|> collect()
					|> col("total_age")
					|> sum()
				"#,
			),
			Value::Int(80),
		),
	]);
}

#[test]
fn test_vectors() {
	helpers::eval_expect_values(&[
		("data::intvec(1, 2, 3) |> data::sum()", Value::Int(6)),
		(
			"typeof(data::intvec(1)) == data::Vector<Int>",
			Value::Bool(true),
		),
	]);
}
