use qry::runtime::Value;

mod helpers;

#[test]
fn test_data_sqlite() {
	helpers::eval_expect_values(&[
		("use data::*
		conn <- connect_sqlite(\":memory:\")
		execute(conn, \"create table test_table (name varchar(255), age integer)\")
		execute(conn, \"insert into test_table (name, age) values ('ruan', 26), ('ruanlater', 27), ('thirdperson', 27)\")
		collect(table(conn, \"test_table\"))
		", Value::Null(())),
	]);
}
