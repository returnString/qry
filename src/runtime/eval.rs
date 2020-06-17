use super::{
	eval_callable, eval_function_decl, Callable, EnvironmentPtr, EvalContext, Exception, Type, Value,
};
use crate::lang::syntax::*;

pub type EvalResult<T> = Result<T, Exception>;

pub fn assign_value(ctx: &EvalContext, name: &str, value: Value) -> EvalResult<Value> {
	ctx.env.borrow_mut().update(name, value.clone());
	Ok(value)
}

fn eval_assign(ctx: &EvalContext, dest: &SyntaxNode, src: &SyntaxNode) -> EvalResult<Value> {
	match &dest.syntax {
		Syntax::Ident(name) => {
			let value = eval(ctx, src)?;
			assign_value(ctx, name, value)
		}
		_ => Err(ctx.exception(&dest.location, "assignment requires an identifier")),
	}
}

fn eval_unop(ctx: &EvalContext, target: &SyntaxNode, op: UnaryOperator) -> EvalResult<Value> {
	let method = &ctx.methods.unops[&op];
	eval_callable(ctx, &**method, &[target.clone()], &[])
}

fn eval_binop(
	ctx: &EvalContext,
	lhs: &SyntaxNode,
	rhs: &SyntaxNode,
	op: BinaryOperator,
) -> EvalResult<Value> {
	match op {
		BinaryOperator::LAssign => eval_assign(ctx, lhs, rhs),
		BinaryOperator::RAssign => eval_assign(ctx, rhs, lhs),
		BinaryOperator::Access => {
			let rhs_ident = if let Syntax::Ident(name) = &rhs.syntax {
				name
			} else {
				return Err(ctx.exception(&rhs.location, "access operator requires an identifier"));
			};

			if let Value::Library(lib_env) = eval(ctx, lhs)? {
				if let Some(val) = lib_env.borrow().get(rhs_ident) {
					Ok(val)
				} else {
					Err(ctx.exception(&rhs.location, format!("not found: {}", rhs_ident)))
				}
			} else {
				Err(ctx.exception(&lhs.location, "access operator requires a library"))
			}
		}
		// TODO: pipes can be expressed as a syntax rewrite pass before eval
		BinaryOperator::Pipe => match &rhs.syntax {
			Syntax::Call {
				target,
				positional_args,
				named_args,
			} => {
				let mut new_args = positional_args.clone();
				new_args.insert(0, lhs.clone());
				eval(
					ctx,
					&SyntaxNode {
						syntax: Syntax::Call {
							target: target.clone(),
							positional_args: new_args,
							named_args: named_args.clone(),
						},
						..rhs.clone()
					},
				)
			}
			_ => Err(ctx.exception(
				&rhs.location,
				"right-hand side of a pipe expression must be a call",
			)),
		},
		_ => {
			let method = &ctx.methods.binops[&op];
			eval_callable(ctx, &**method, &[lhs.clone(), rhs.clone()], &[])
		}
	}
}

fn resolve_lib(
	ctx: &EvalContext,
	parent_node: &SyntaxNode,
	from: &[String],
) -> Result<EnvironmentPtr, Exception> {
	let mut current_env = ctx.library_env.clone();
	for name in from {
		let val = { current_env.borrow().get(name) };
		if let Some(lib_value) = val {
			if let Value::Library(lib_env) = lib_value {
				current_env = lib_env;
			} else {
				return Err(ctx.exception(&parent_node.location, "expected a library"));
			}
		} else {
			return Err(ctx.exception(&parent_node.location, "expected a library"));
		}
	}
	Ok(current_env)
}

fn eval_import(
	ctx: &EvalContext,
	parent_node: &SyntaxNode,
	from: &[String],
	import: &Import,
) -> EvalResult<Value> {
	let lib_env = resolve_lib(ctx, parent_node, from)?;

	match import {
		Import::Named(names) => {
			for name in names {
				if let Some(val) = lib_env.borrow().get(name) {
					ctx.env.borrow_mut().update(name, val);
				} else {
					return Err(ctx.exception(&parent_node.location, format!("not found: {}", name)));
				}
			}
		}
		Import::Wildcard => lib_env.borrow().copy_to(&mut ctx.env.borrow_mut()),
	}

	Ok(Value::Null(()))
}

pub fn eval_multi(ctx: &EvalContext, exprs: &[SyntaxNode]) -> EvalResult<Value> {
	let mut ret = Value::Null(());
	for expr in exprs {
		ret = eval(ctx, expr)?;
	}
	Ok(ret)
}

pub fn eval(ctx: &EvalContext, node: &SyntaxNode) -> EvalResult<Value> {
	match &node.syntax {
		Syntax::Int(val) => Ok(Value::Int(*val)),
		Syntax::Float(val) => Ok(Value::Float(*val)),
		Syntax::String(val) => Ok(Value::String(val.clone().into_boxed_str())),
		Syntax::Bool(val) => Ok(Value::Bool(*val)),
		Syntax::Null => Ok(Value::Null(())),
		Syntax::BinaryOp { lhs, rhs, op } => eval_binop(ctx, lhs, rhs, *op),
		Syntax::UnaryOp { target, op } => eval_unop(ctx, target, *op),
		Syntax::Interpolate(_) => Err(ctx.exception(
			&node.location,
			"interpolation not supported in regular code",
		)),
		Syntax::Function {
			header,
			params,
			return_type,
			body,
		} => eval_function_decl(ctx, &node.location, header, params, return_type, body),
		Syntax::Use { from, import } => eval_import(ctx, node, from, import),
		Syntax::Ident(name) => {
			if let Some(val) = ctx.env.borrow().get(name) {
				Ok(val)
			} else {
				Err(ctx.exception(&node.location, format!("not found: {}", name)))
			}
		}
		Syntax::Call {
			target,
			positional_args,
			named_args,
		} => {
			let named_args = named_args
				.iter()
				.map(|(n, s)| (n.as_ref(), s.clone()))
				.collect::<Vec<_>>();

			match eval(ctx, target)? {
				Value::Builtin(builtin) => eval_callable(ctx, &*builtin, positional_args, &named_args),
				Value::Function(func) => eval_callable(ctx, &*func, positional_args, &named_args),
				Value::Method(method) => eval_callable(ctx, &*method, positional_args, &named_args),
				_ => Err(ctx.exception(&node.location, "target is not callable")),
			}
		}
		Syntax::Switch { target, cases } => {
			let target_val = eval(ctx, target)?;
			let mut ret = Value::Null(());
			let eq_method = &ctx.methods.binops[&BinaryOperator::Equal];
			for case in cases {
				let case_val = eval(ctx, &case.expr)?;
				let eq_val = eq_method.call(ctx, &[target_val.clone(), case_val], &[])?;

				if eq_val.as_bool() {
					ret = eval(ctx, &case.returns)?;
					break;
				}
			}
			Ok(ret)
		}
		Syntax::Index { target, keys } => {
			let mut args = vec![eval(ctx, target)?];
			args.extend(
				keys
					.iter()
					.map(|k| eval(ctx, k))
					.collect::<Result<Vec<_>, _>>()?,
			);

			let ret = ctx.methods.index.call(ctx, &args, &[])?;
			Ok(ret)
		}
		Syntax::GenericInstantiation { target, type_args } => {
			let generic_lookup = match eval(ctx, target)? {
				Value::Type(t) => match t {
					Type::Native(d) => match d.generic_lookup {
						Some(l) => Ok(l),
						_ => Err(ctx.exception(&target.location, "native type is not generic")),
					},
					_ => Err(ctx.exception(&target.location, "unsupported type for generics")),
				},
				_ => Err(ctx.exception(&target.location, "unsupported value type for generics")),
			}?;

			let types = type_args
				.iter()
				.map(|a| match eval(ctx, a)? {
					Value::Type(t) => Ok(t),
					_ => Err(ctx.exception(&a.location, "expected a type")),
				})
				.collect::<EvalResult<Vec<_>>>()?;

			Ok(Value::Type(generic_lookup(ctx, &types)?))
		}
	}
}
