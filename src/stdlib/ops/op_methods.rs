use crate::lang::{BinaryOperator, UnaryOperator};
use crate::runtime::{
	Builtin, Environment, EnvironmentPtr, Method, RuntimeMethods, Signature, Type, Value,
};
use std::collections::HashMap;
use std::rc::Rc;

pub fn create() -> (RuntimeMethods, EnvironmentPtr) {
	let env = Environment::new("ops");
	let binops = init_binops();
	let unops = init_unops();
	let to_string = Method::new("to_string", &["val"], Some(Type::String), None);
	init_to_string(&to_string);

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
			binops,
			unops,
		},
		env,
	)
}

macro_rules! binop {
	($lhs_type: ident, $rhs_type: ident, $return_type: ident, $builder: expr) => {
		Builtin::new(
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
		let assoc = |op, func| $map.get(&op).unwrap().register(func);
		assoc(
			BinaryOperator::Equal,
			binop!($lhs_type, $rhs_type, Bool, |a, b| (a as $native_type
				== (b as $native_type))),
		);
		assoc(
			BinaryOperator::NotEqual,
			binop!($lhs_type, $rhs_type, Bool, |a, b| (a as $native_type
				!= (b as $native_type))),
		);
	};
}

macro_rules! numeric_binops {
	($map: expr, $lhs_type: ident, $rhs_type: ident, $return_type: ident, $native_type: ty) => {
		let assoc = |op, func| $map.get(&op).unwrap().register(func);
		assoc(
			BinaryOperator::Add,
			binop!($lhs_type, $rhs_type, $return_type, |a, b| (a
				as $native_type)
				+ (b as $native_type)),
		);
		assoc(
			BinaryOperator::Sub,
			binop!($lhs_type, $rhs_type, $return_type, |a, b| (a
				as $native_type)
				- (b as $native_type)),
		);
		assoc(
			BinaryOperator::Mul,
			binop!($lhs_type, $rhs_type, $return_type, |a, b| (a
				as $native_type)
				* (b as $native_type)),
		);
		assoc(
			BinaryOperator::Div,
			binop!($lhs_type, $rhs_type, $return_type, |a, b| (a
				as $native_type)
				/ (b as $native_type)),
		);
		assoc(
			BinaryOperator::Lt,
			binop!($lhs_type, $rhs_type, Bool, |a, b| (a as $native_type)
				< (b as $native_type)),
		);
		assoc(
			BinaryOperator::Lte,
			binop!($lhs_type, $rhs_type, Bool, |a, b| (a as $native_type)
				<= (b as $native_type)),
		);
		assoc(
			BinaryOperator::Gt,
			binop!($lhs_type, $rhs_type, Bool, |a, b| (a as $native_type)
				> (b as $native_type)),
		);
		assoc(
			BinaryOperator::Gte,
			binop!($lhs_type, $rhs_type, Bool, |a, b| (a as $native_type)
				>= (b as $native_type)),
		);

		equality_ops!($map, $lhs_type, $rhs_type, $native_type);
	};
}

macro_rules! unop {
	($target_type: ident, $return_type: ident, $builder: expr) => {
		Builtin::new(
			Signature::returning(&Type::$return_type).param("a", &Type::$target_type),
			|_, args, _| match &args[0] {
				Value::$target_type(a) => Ok(Value::$return_type($builder(a.clone()))),
				_ => unreachable!(),
			},
			)
	};
}

fn init_to_string(to_string: &Method) {
	to_string.register(unop!(Null, String, |_| "null".into()));
	to_string.register(unop!(String, String, |a| a));
	to_string.register(unop!(Int, String, |a: i64| a.to_string().into_boxed_str()));
	to_string.register(unop!(Float, String, |a: f64| format!("{:?}", a).into_boxed_str()));
	to_string.register(unop!(Bool, String, |a: bool| a
		.to_string()
		.into_boxed_str()));
	to_string.register(Builtin::new(
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
						.map(|p| format!("{}: {:?}", p.name, p.param_type))
						.collect::<Vec<_>>()
						.join(", ");
					format!("({}) -> {:?}", param_string, s.return_type)
				})
				.collect::<Vec<_>>()
				.join("\n");

			Ok(Value::String(
				format!("method '{}' with impls:\n{}", method.name(), signatures).into_boxed_str(),
			))
		},
	));
	to_string.register(Builtin::new(
		Signature::returning(&Type::String).param("obj", &Type::Type),
		|_, args, _| {
			let type_val = args[0].as_type();
			Ok(Value::String(type_val.name().into()))
		},
	));
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
	and.register(binop!(Bool, Bool, Bool, |a, b| a && b));
	or.register(binop!(Bool, Bool, Bool, |a, b| a || b));

	add.register(binop!(String, String, String, |a, b| format!("{}{}", a, b)
		.into_boxed_str()));
	equal.register(binop!(String, String, Bool, |a, b| a == b));
	not_equal.register(binop!(String, String, Bool, |a, b| a != b));

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

	negate.register(unop!(Bool, Bool, |a: bool| !a));

	minus.register(unop!(Int, Int, |a: i64| -a));
	minus.register(unop!(Float, Float, |a: f64| -a));

	m
}
