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
	]);
}
