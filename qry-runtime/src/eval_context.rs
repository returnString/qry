use super::{Environment, Exception, Method, Value};
use crate::stdlib;
use qry_lang::{BinaryOperator, SourceLocation, UnaryOperator};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct StackFrame {
	pub name: String,
	pub location: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct RuntimeMethods {
	pub to_string: Rc<Method>,
	pub index: Rc<Method>,
	pub binops: HashMap<BinaryOperator, Rc<Method>>,
	pub unops: HashMap<UnaryOperator, Rc<Method>>,
}

pub struct EvalStackFrameScope<'a> {
	ctx: &'a EvalContext,
}

impl<'a> EvalStackFrameScope<'a> {
	pub fn new(ctx: &'a EvalContext, name: &str, location: &SourceLocation) -> Self {
		ctx.callstack.borrow_mut().push(StackFrame {
			name: name.to_owned(),
			location: location.clone(),
		});

		Self { ctx }
	}
}

impl Drop for EvalStackFrameScope<'_> {
	fn drop(&mut self) {
		self.ctx.callstack.borrow_mut().pop();
	}
}

#[derive(Debug, Clone)]
pub struct EvalContext {
	pub env: Rc<Environment>,
	pub library_env: Rc<Environment>,
	pub methods: Rc<RuntimeMethods>,
	pub callstack: Rc<RefCell<Vec<StackFrame>>>,
}

impl EvalContext {
	pub fn new_with_stdlib() -> Self {
		let global_env_ptr = Environment::new("global");
		let library_env_ptr = Environment::new("libraries");

		let add_lib = |env: Rc<Environment>, add_to_global| {
			let lib_val = Value::Library(env.clone());
			library_env_ptr.update(env.name(), lib_val.clone());

			if add_to_global {
				env.copy_to(&global_env_ptr);
			}

			global_env_ptr.update(env.name(), lib_val);
		};

		let (ops_methods, ops_env) = stdlib::ops::create();
		add_lib(ops_env, false);
		add_lib(stdlib::core::env(&ops_methods), true);
		add_lib(stdlib::data::env(&ops_methods), false);

		EvalContext {
			env: global_env_ptr,
			library_env: library_env_ptr,
			methods: Rc::new(ops_methods),
			callstack: Rc::default(),
		}
	}

	pub fn child(&self, env: Rc<Environment>) -> EvalContext {
		EvalContext {
			env,
			..self.clone()
		}
	}

	#[must_use]
	pub fn with_stack_frame(&self, name: &str, location: &SourceLocation) -> EvalStackFrameScope {
		EvalStackFrameScope::new(self, name, location)
	}

	pub fn exception<S: Into<String>>(&self, location: &SourceLocation, message: S) -> Exception {
		Exception {
			message: message.into(),
			location: location.clone(),
			stack: self.callstack.borrow().clone(),
		}
	}
}
