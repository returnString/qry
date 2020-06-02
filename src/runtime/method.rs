use super::{Callable, EvalContext, EvalError, EvalResult, Parameter, Signature, Type, Value};
use crate::lang::SourceLocation;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Method {
	name: String,
	signature: Signature,
	impls: RefCell<HashMap<Vec<Type>, Rc<dyn Callable>>>,
	fixed_return_type: Option<Type>,
	default_impl: Option<Rc<dyn Callable>>,
}

impl Method {
	pub fn new(
		name: &str,
		dispatch_param_names: &[&str],
		fixed_return_type: Option<Type>,
		default_impl: Option<Rc<dyn Callable>>,
	) -> Rc<Self> {
		let params = dispatch_param_names
			.iter()
			.map(|n| Parameter {
				name: (*n).to_string(),
				param_type: Type::Any,
			})
			.collect::<Vec<_>>();

		Rc::new(Self {
			name: name.into(),
			fixed_return_type: fixed_return_type.clone(),
			signature: Signature {
				// FIXME: need a better placeholder for methods with varying return types
				return_type: fixed_return_type.unwrap_or(Type::Any),
				params,
				trailing_type: None,
				named_trailing_type: None,
			},
			impls: Default::default(),
			default_impl,
		})
	}

	pub fn name(&self) -> &str {
		&self.name
	}

	pub fn supported_signatures(&self) -> Vec<Signature> {
		self
			.impls
			.borrow()
			.iter()
			.map(|(_, v)| v.signature().clone())
			.collect()
	}

	fn get_sig_key(&self, types: &[Type]) -> Vec<Type> {
		types[..self.signature.params.len()].to_owned()
	}

	pub fn register(&self, callable: Rc<dyn Callable>) {
		if let Some(return_type) = &self.fixed_return_type {
			assert_eq!(*return_type, callable.signature().return_type)
		}

		let key = self.get_sig_key(
			&callable
				.signature()
				.params
				.iter()
				.map(|p| p.param_type.clone())
				.collect::<Vec<_>>(),
		);
		self.impls.borrow_mut().insert(key, callable);
	}

	pub fn resolve(&self, types: &[Type]) -> Option<Rc<dyn Callable>> {
		let key = self.get_sig_key(types);
		self.impls.borrow().get(&key).cloned()
	}
}

impl Callable for Method {
	fn signature(&self) -> &Signature {
		&self.signature
	}

	fn source_location(&self) -> &SourceLocation {
		&SourceLocation::Native
	}

	fn name(&self) -> &str {
		&self.name
	}

	fn call(
		&self,
		ctx: &EvalContext,
		args: &[Value],
		named_trailing: &[(&str, Value)],
	) -> EvalResult<Value> {
		let arg_types = args.iter().map(|a| a.runtime_type()).collect::<Vec<_>>();

		if let Some(callable) = self.resolve(&arg_types) {
			callable.call(ctx, args, named_trailing)
		} else if let Some(callable) = &self.default_impl {
			callable.call(ctx, args, named_trailing)
		} else {
			Err(EvalError::MethodNotImplemented)
		}
	}
}
