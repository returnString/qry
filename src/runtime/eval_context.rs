use super::{Environment, EnvironmentPtr, Method, Value};
use crate::lang::{BinaryOperator, UnaryOperator};
use crate::stdlib;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct RuntimeMethods {
	pub to_string: Rc<Method>,
	pub index: Rc<Method>,
	pub binops: HashMap<BinaryOperator, Rc<Method>>,
	pub unops: HashMap<UnaryOperator, Rc<Method>>,
}

#[derive(Debug, Clone)]
pub struct EvalContext {
	pub env: EnvironmentPtr,
	pub library_env: EnvironmentPtr,
	pub methods: Rc<RuntimeMethods>,
}

impl EvalContext {
	pub fn new_with_stdlib() -> Self {
		let global_env_ptr = Environment::new("global");
		let library_env_ptr = Environment::new("libraries");

		let add_lib = |env_ptr: EnvironmentPtr, add_to_global| {
			let lib_val = Value::Library(env_ptr.clone());
			let env = env_ptr.borrow();
			library_env_ptr.borrow_mut().update(env.name(), lib_val);

			if add_to_global {
				env.copy_to(&mut global_env_ptr.borrow_mut());
			}
		};

		let (ops_methods, ops_env) = stdlib::ops::create();
		add_lib(ops_env, false);
		add_lib(stdlib::core::env(&ops_methods), true);
		add_lib(stdlib::data::env(&ops_methods), false);

		EvalContext {
			env: global_env_ptr,
			library_env: library_env_ptr,
			methods: Rc::new(ops_methods),
		}
	}

	pub fn child(&self, env_ptr: EnvironmentPtr) -> EvalContext {
		EvalContext {
			env: env_ptr,
			..self.clone()
		}
	}
}
