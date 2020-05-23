use crate::lang::{BinaryOperator, UnaryOperator};
use crate::runtime::{
	Builtin, Environment, EnvironmentPtr, Method, MethodPtr, Parameter, Signature, Type, Value,
};
use std::collections::HashMap;

pub fn ops_module() -> EnvironmentPtr {
	let env = Environment::new("ops");

	{
		let mut env = env.borrow_mut();

		BINOP_LOOKUP.with(|m| {
			for (k, v) in m {
				if let Some(name) = k.name() {
					env.update(name, Value::Method(v.clone()));
				}
			}
		});

		UNOP_LOOKUP.with(|m| {
			for (k, v) in m {
				if let Some(name) = k.name() {
					env.update(name, Value::Method(v.clone()));
				}
			}
		});
	}

	env
}

macro_rules! binop {
	($lhs_type: ident, $rhs_type: ident, $return_type: ident, $builder: expr) => {
		Builtin::new(
			Signature {
				params: vec![
					Parameter {
						name: "a".to_string(),
						param_type: Type::$lhs_type,
					},
					Parameter {
						name: "b".to_string(),
						param_type: Type::$rhs_type,
					},
				],
				return_type: Type::$return_type,
				with_trailing: false,
				with_named_trailing: false,
			},
			|args| match (&args[0], &args[1]) {
				(Value::$lhs_type(a), Value::$rhs_type(b)) => {
					Value::$return_type($builder(a.clone(), b.clone()))
				}
				_ => unreachable!(),
			},
			)
	};
}

macro_rules! equality_ops {
	($map: expr, $lhs_type: ident, $rhs_type: ident, $native_type: ty) => {
		let assoc = |op, func| $map.get(&op).unwrap().borrow_mut().register(func);
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
		let assoc = |op, func| $map.get(&op).unwrap().borrow_mut().register(func);
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
			Signature {
				params: vec![Parameter {
					name: "a".to_string(),
					param_type: Type::$target_type,
				}],
				return_type: Type::$return_type,
				with_trailing: false,
				with_named_trailing: false,
			},
			|args| match &args[0] {
				Value::$target_type(a) => Value::$return_type($builder(a.clone())),
				_ => unreachable!(),
			},
			)
	};
}

thread_local! {
	#[allow(clippy::float_cmp)] // this is invoked by the Float == Float method
	pub static BINOP_LOOKUP: HashMap<BinaryOperator, MethodPtr> = {
		let mut m = HashMap::new();
		let mut new_binop = |op| {
			let method = Method::new(&["a", "b"]);
			m.insert(op, method.clone());
			method
		};

		let add_method = new_binop(BinaryOperator::Add);
		new_binop(BinaryOperator::Sub);
		new_binop(BinaryOperator::Mul);
		new_binop(BinaryOperator::Div);
		new_binop(BinaryOperator::Equal);
		new_binop(BinaryOperator::NotEqual);
		new_binop(BinaryOperator::Lt);
		new_binop(BinaryOperator::Lte);
		new_binop(BinaryOperator::Gt);
		new_binop(BinaryOperator::Gte);

		{
			numeric_binops!(m, Int, Int, Int, i64);
			numeric_binops!(m, Float, Float, Float, f64);
			numeric_binops!(m, Int, Float, Float, f64);
			numeric_binops!(m, Float, Int, Float, f64);

			equality_ops!(m, Bool, Bool, bool);

			let mut add_method = add_method.borrow_mut();
			add_method.register(binop!(String, String, String, |a, b| format!("{}{}", a, b).into_boxed_str()));
		}

		m
	};

	pub static UNOP_LOOKUP: HashMap<UnaryOperator, MethodPtr> = {
		let mut m = HashMap::new();
		let mut new_unop = |op| {
			let method = Method::new(&["a"]);
			m.insert(op, method.clone());
			method
		};

		let negate = new_unop(UnaryOperator::Negate);
		let minus = new_unop(UnaryOperator::Minus);

		let mut negate = negate.borrow_mut();
		negate.register(unop!(Bool, Bool, |a: bool| !a));

		let mut minus = minus.borrow_mut();
		minus.register(unop!(Int, Int, |a: i64| -a));
		minus.register(unop!(Float, Float, |a: f64| -a));

		m
	}
}
