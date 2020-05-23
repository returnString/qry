pub trait ConnectionImpl {
	fn collect(&self, sql: &str);
}

#[derive(Debug)]
pub struct Connection {}

impl Drop for Connection {
	fn drop(&mut self) {
		println!("connection is now dead")
	}
}
