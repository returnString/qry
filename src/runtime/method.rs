use super::{Callable, EvalContext, EvalError, EvalResult, Parameter, Signature, Type, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone)]
pub struct Method {
	name: String,
	signature: Signature,
	impls: HashMap<Vec<Type>, Rc<dyn Callable>>,
	fixed_return_type: Option<Type>,
	default_impl: Option<Rc<dyn Callable>>,
}

pub type MethodPtr = Rc<RefCell<Method>>;

impl Method {
	pub fn new(
		name: &str,
		dispatch_param_names: &[&str],
		fixed_return_type: Option<Type>,
		default_impl: Option<Rc<dyn Callable>>,
	) -> MethodPtr {
		let params = dispatch_param_names
			.iter()
			.map(|n| Parameter {
				name: (*n).to_string(),
				param_type: Type::MethodDispatchPlaceholder,
			})
			.collect::<Vec<_>>();

		Rc::new(RefCell::new(Self {
			name: name.into(),
			fixed_return_type: fixed_return_type.clone(),
			signature: Signature {
				// FIXME: need a better placeholder for methods with varying return types
				return_type: fixed_return_type.unwrap_or(Type::Null),
				params,
				with_trailing: false,
				with_named_trailing: false,
			},
			impls: Default::default(),
			default_impl,
		}))
	}

	pub fn name(&self) -> &str {
		&self.name
	}

	fn get_sig_key(&self, types: &[Type]) -> Vec<Type> {
		types[..self.signature.params.len()].to_owned()
	}

	pub fn register(&mut self, callable: Rc<dyn Callable>) {
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
		self.impls.insert(key, callable);
	}

	pub fn resolve(&self, types: &[Type]) -> Option<Rc<dyn Callable>> {
		let key = self.get_sig_key(types);
		self.impls.get(&key).cloned()
	}
}

impl std::fmt::Debug for Method {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		f.debug_struct("Method")
			.field("signature", &self.signature)
			.finish()
	}
}

impl Callable for Method {
	fn signature(&self) -> &Signature {
		&self.signature
	}

	fn call(
		&self,
		ctx: &EvalContext,
		args: &[(&String, &Value)],
		named_trailing: &[(&String, &Value)],
	) -> EvalResult {
		let arg_types = args
			.iter()
			.map(|(_, a)| a.runtime_type())
			.collect::<Vec<_>>();

		if let Some(callable) = self.resolve(&arg_types) {
			callable.call(ctx, args, named_trailing)
		} else if let Some(callable) = &self.default_impl {
			callable.call(ctx, args, named_trailing)
		} else {
			Err(EvalError::MethodNotImplemented)
		}
	}
}
