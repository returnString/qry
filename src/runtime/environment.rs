use super::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Default, Debug)]
pub struct Environment {
	name: String,
	state: HashMap<String, Value>,
}

pub type EnvironmentPtr = Rc<RefCell<Environment>>;

impl Environment {
	pub fn new(name: &str) -> EnvironmentPtr {
		Rc::new(RefCell::new(Environment {
			name: name.to_string(),
			state: HashMap::new(),
		}))
	}

	pub fn name(&self) -> &str {
		&self.name
	}

	pub fn update(&mut self, name: &str, val: Value) {
		self.state.insert(name.to_string(), val.clone());
	}

	pub fn get(&self, name: &str) -> Option<Value> {
		if let Some(val) = self.state.get(name) {
			Some(val.clone())
		} else {
			None
		}
	}

	pub fn copy_to(&self, target: &mut Environment) {
		for (k, v) in &self.state {
			target.update(k, v.clone());
		}
	}

	pub fn child(&self, name: &str) -> EnvironmentPtr {
		let env = Self::new(name);
		self.copy_to(&mut env.borrow_mut());
		env
	}
}
