use crate::lang::BinaryOperator;
use crate::runtime::{
	Builtin, Environment, EnvironmentPtr, Method, MethodPtr, Parameter, Signature, Type, Value,
};
use std::collections::HashMap;

pub fn ops_module() -> EnvironmentPtr {
	let env = Environment::new("ops");

	{
		let mut env = env.borrow_mut();

		BINOP_LOOKUP.with(|m| {
			env.update(
				"add",
				Value::Method(m.get(&BinaryOperator::Add).unwrap().clone()),
			);
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

macro_rules! numeric_binops {
	($map: expr, $lhs_type: ident, $rhs_type: ident, $return_type: ident, $native_type: ident) => {
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
	};
}

thread_local! {
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

		{
			numeric_binops!(m, Int, Int, Int, i64);
			numeric_binops!(m, Float, Float, Float, f64);
			numeric_binops!(m, Int, Float, Float, f64);
			numeric_binops!(m, Float, Int, Float, f64);

			let mut add_method = add_method.borrow_mut();
			add_method.register(binop!(String, String, String, |a, b| format!("{}{}", a, b).into_boxed_str()));
		}

		m
	};
}
