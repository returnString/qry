use super::{
	Builtin, BuiltinFunc, Callable, Method, NativeGenericType, NativeType, Signature, Type, Value,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::panic::Location;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Environment {
	name: String,
	state: RefCell<HashMap<String, Value>>,
}

impl Environment {
	pub fn new(name: &str) -> Rc<Self> {
		Rc::new(Self {
			name: name.to_string(),
			state: RefCell::new(HashMap::new()),
		})
	}

	pub fn name(&self) -> &str {
		&self.name
	}

	pub fn update(&self, name: &str, val: Value) {
		self.state.borrow_mut().insert(name.to_string(), val);
	}

	pub fn get(&self, name: &str) -> Option<Value> {
		if let Some(val) = self.state.borrow().get(name) {
			Some(val.clone())
		} else {
			None
		}
	}

	pub fn copy_to(&self, target: &Environment) {
		for (k, v) in self.state.borrow().iter() {
			target.update(k, v.clone());
		}
	}

	pub fn child(&self, name: &str) -> Rc<Self> {
		let env = Self::new(name);
		self.copy_to(&env);
		env
	}

	pub fn define_native_type<T: 'static + NativeType>(&self) -> Type {
		let native_type = Type::new_native::<T>();
		self.update(T::name(), Value::Type(native_type.clone()));
		native_type
	}

	pub fn define_native_generic_type<T: 'static + NativeGenericType>(&self) -> Type {
		let native_type = Type::new_native_generic::<T>();
		self.update(T::name(), Value::Type(native_type.clone()));
		native_type
	}

	#[track_caller]
	pub fn define_builtin(&self, name: &str, signature: Signature, func: BuiltinFunc) {
		let builtin = Value::Builtin(Builtin::new(
			&format!("{}::{}", self.name, name),
			signature,
			Location::caller().into(),
			func,
		));
		self.update(name, builtin);
	}

	pub fn define_method(
		&self,
		name: &str,
		dispatch_param_names: &[&str],
		fixed_return_type: Option<Type>,
		default_impl: Option<Rc<dyn Callable>>,
	) -> Rc<Method> {
		let method = Method::new(name, dispatch_param_names, fixed_return_type, default_impl);
		self.update(name, Value::Method(method.clone()));
		method
	}
}
