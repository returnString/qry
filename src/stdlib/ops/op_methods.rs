use crate::lang::{BinaryOperator, UnaryOperator};
use crate::runtime::{Environment, EnvironmentPtr, Method, RuntimeMethods, Signature, Type, Value};
use std::collections::HashMap;
use std::rc::Rc;

pub fn create() -> (RuntimeMethods, EnvironmentPtr) {
	let env = Environment::new("ops");
	let binops = init_binops();
	let unops = init_unops();
	let to_string = Method::new("to_string", &["val"], Some(Type::String), None);
	let index = Method::new("index", &["container", "key"], None, None);
	init_to_string(&to_string);
	init_index(&index);

	{
		let mut env = env.borrow_mut();

		for v in binops.values() {
			env.update(v.name(), Value::Method(v.clone()));
		}

		for v in unops.values() {
			env.update(v.name(), Value::Method(v.clone()));
		}
	}

	(
		RuntimeMethods {
			to_string,
			index,
			binops,
			unops,
		},
		env,
	)
}

macro_rules! binop {
	($method: expr, $lhs_type: ident, $rhs_type: ident, $return_type: ident, $builder: expr) => {
		$method.register_builtin(
			Signature::returning(&Type::$return_type)
				.param("a", &Type::$lhs_type)
				.param("b", &Type::$rhs_type),
			|_, args, _| match (&args[0], &args[1]) {
				(Value::$lhs_type(a), Value::$rhs_type(b)) => {
					Ok(Value::$return_type($builder(a.clone(), b.clone())))
				}
				_ => unreachable!(),
			},
			)
	};
}

macro_rules! equality_ops {
	($map: expr, $lhs_type: ident, $rhs_type: ident, $native_type: ty) => {
		binop!(
			$map[&BinaryOperator::Equal],
			$lhs_type,
			$rhs_type,
			Bool,
			|a, b| (a as $native_type == (b as $native_type))
		);

		binop!(
			$map[&BinaryOperator::NotEqual],
			$lhs_type,
			$rhs_type,
			Bool,
			|a, b| (a as $native_type != (b as $native_type))
		);
	};
}

macro_rules! numeric_binops {
	($map: expr, $lhs_type: ident, $rhs_type: ident, $return_type: ident, $native_type: ty) => {
		binop!(
			$map[&BinaryOperator::Add],
			$lhs_type,
			$rhs_type,
			$return_type,
			|a, b| (a as $native_type) + (b as $native_type)
		);
		binop!(
			$map[&BinaryOperator::Sub],
			$lhs_type,
			$rhs_type,
			$return_type,
			|a, b| (a as $native_type) - (b as $native_type)
		);
		binop!(
			$map[&BinaryOperator::Mul],
			$lhs_type,
			$rhs_type,
			$return_type,
			|a, b| (a as $native_type) * (b as $native_type)
		);
		binop!(
			$map[&BinaryOperator::Div],
			$lhs_type,
			$rhs_type,
			$return_type,
			|a, b| (a as $native_type) / (b as $native_type)
		);
		binop!(
			$map[&BinaryOperator::Lt],
			$lhs_type,
			$rhs_type,
			Bool,
			|a, b| (a as $native_type) < (b as $native_type)
		);
		binop!(
			$map[&BinaryOperator::Lte],
			$lhs_type,
			$rhs_type,
			Bool,
			|a, b| (a as $native_type) <= (b as $native_type)
		);
		binop!(
			$map[&BinaryOperator::Gt],
			$lhs_type,
			$rhs_type,
			Bool,
			|a, b| (a as $native_type) > (b as $native_type)
		);
		binop!(
			$map[&BinaryOperator::Gte],
			$lhs_type,
			$rhs_type,
			Bool,
			|a, b| (a as $native_type) >= (b as $native_type)
		);

		equality_ops!($map, $lhs_type, $rhs_type, $native_type);
	};
}

macro_rules! unop {
	($method: expr, $target_type: ident, $return_type: ident, $builder: expr) => {
		$method.register_builtin(
			Signature::returning(&Type::$return_type).param("a", &Type::$target_type),
			|_, args, _| match &args[0] {
				Value::$target_type(a) => Ok(Value::$return_type($builder(a.clone()))),
				_ => unreachable!(),
			},
			)
	};
}

fn init_to_string(to_string: &Method) {
	unop!(to_string, Null, String, |_| "null".into());
	unop!(to_string, String, String, |a| a);
	unop!(to_string, Int, String, |a: i64| a
		.to_string()
		.into_boxed_str());
	unop!(to_string, Float, String, |a: f64| format!("{:?}", a)
		.into_boxed_str());
	unop!(to_string, Bool, String, |a: bool| a
		.to_string()
		.into_boxed_str());
	to_string.register_builtin(
		Signature::returning(&Type::String).param("obj", &Type::Method),
		|_, args, _| {
			let method = args[0].as_method();
			let signatures = method
				.supported_signatures()
				.iter()
				.map(|s| {
					let param_string = s
						.params
						.iter()
						.map(|p| format!("{}: {}", p.name, p.param_type.name()))
						.collect::<Vec<_>>()
						.join(", ");

					format!("({}) -> {}", param_string, s.return_type.name())
				})
				.collect::<Vec<_>>()
				.join("\n");

			Ok(Value::String(
				format!("method '{}' with impls:\n{}", method.name(), signatures).into_boxed_str(),
			))
		},
	);
	to_string.register_builtin(
		Signature::returning(&Type::String).param("obj", &Type::Type),
		|_, args, _| {
			let type_val = args[0].as_type();
			Ok(Value::String(type_val.name().into()))
		},
	);
}

fn init_index(index: &Method) {
	index.register_builtin(
		Signature::returning(&Type::Any)
			.param("list", &Type::List)
			.param("index", &Type::Int),
		|_, args, _| {
			let list = args[0].as_list();
			let index = args[1].as_int() as usize;
			Ok(list[index].clone())
		},
	);
}

#[allow(clippy::float_cmp)] // this is invoked by the Float == Float method
fn init_binops() -> HashMap<BinaryOperator, Rc<Method>> {
	let mut m = HashMap::new();
	let mut new_binop = |name, op| {
		let method = Method::new(name, &["a", "b"], None, None);
		m.insert(op, method.clone());
		method
	};

	let add = new_binop("add", BinaryOperator::Add);
	new_binop("sub", BinaryOperator::Sub);
	new_binop("mul", BinaryOperator::Mul);
	new_binop("div", BinaryOperator::Div);
	let equal = new_binop("equal", BinaryOperator::Equal);
	let not_equal = new_binop("not_equal", BinaryOperator::NotEqual);
	new_binop("lt", BinaryOperator::Lt);
	new_binop("lte", BinaryOperator::Lte);
	new_binop("gt", BinaryOperator::Gt);
	new_binop("gte", BinaryOperator::Gte);
	let and = new_binop("and", BinaryOperator::And);
	let or = new_binop("or", BinaryOperator::Or);

	numeric_binops!(m, Int, Int, Int, i64);
	numeric_binops!(m, Float, Float, Float, f64);
	numeric_binops!(m, Int, Float, Float, f64);
	numeric_binops!(m, Float, Int, Float, f64);

	equality_ops!(m, Bool, Bool, bool);
	binop!(and, Bool, Bool, Bool, |a, b| a && b);
	binop!(or, Bool, Bool, Bool, |a, b| a || b);

	binop!(add, String, String, String, |a, b| format!("{}{}", a, b)
		.into_boxed_str());
	binop!(equal, String, String, Bool, |a, b| a == b);
	binop!(not_equal, String, String, Bool, |a, b| a != b);

	m
}

fn init_unops() -> HashMap<UnaryOperator, Rc<Method>> {
	let mut m = HashMap::new();
	let mut new_unop = |name, op| {
		let method = Method::new(name, &["a"], None, None);
		m.insert(op, method.clone());
		method
	};

	let negate = new_unop("negate", UnaryOperator::Negate);
	let minus = new_unop("minus", UnaryOperator::Minus);

	unop!(negate, Bool, Bool, |a: bool| !a);

	unop!(minus, Int, Int, |a: i64| -a);
	unop!(minus, Float, Float, |a: f64| -a);

	m
}
