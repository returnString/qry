use super::{
	Builtin, BuiltinFunc, Callable, Method, NativeGenericType, NativeType, Signature, Type, Value,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::panic::Location;
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
		self.state.insert(name.to_string(), val);
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

	pub fn define_native_type<T: 'static + NativeType>(&mut self) -> Type {
		let native_type = Type::new_native::<T>();
		self.update(T::name(), Value::Type(native_type.clone()));
		native_type
	}

	pub fn define_native_generic_type<T: 'static + NativeGenericType>(&mut self) -> Type {
		let native_type = Type::new_native_generic::<T>();
		self.update(T::name(), Value::Type(native_type.clone()));
		native_type
	}

	#[track_caller]
	pub fn define_builtin(&mut self, name: &str, signature: Signature, func: BuiltinFunc) {
		let builtin = Value::Builtin(Builtin::new(
			&format!("{}::{}", self.name, name),
			signature,
			Location::caller().into(),
			func,
		));
		self.update(name, builtin);
	}

	pub fn define_method(
		&mut self,
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
