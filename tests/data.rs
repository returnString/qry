use qry::runtime::Value;

pub mod helpers;

const TABLE_BOOTSTRAP: &str = "
use data::*
conn <- connect_sqlite(\":memory:\")
execute(conn, \"create table test_table (name varchar(255), age integer)\")
execute(conn, \"insert into test_table (name, age) values ('ruan', 26), ('ruanlater', 27), ('thirdperson', 27), ('ancient one', null)\")
";

fn with_table_bootstrap(query: &str) -> String {
	format!("{}\n{}", TABLE_BOOTSTRAP, query)
}

#[test]
fn test_data_sqlite() {
	helpers::eval_expect_values(&[
		(
			&with_table_bootstrap("num_rows(collect(table(conn, \"test_table\")))"),
			Value::Int(4),
		),
		(
			&with_table_bootstrap(
				"num_rows(collect(filter(table(conn, \"test_table\"), name == \"ruan\")))",
			),
			Value::Int(1),
		),
	]);
}
