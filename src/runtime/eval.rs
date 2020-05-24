use super::{eval_callable, eval_function, Callable, Environment, EnvironmentPtr, Value};
use crate::lang::syntax::*;
use crate::stdlib;

#[derive(Debug, PartialEq, Eq)]
pub enum EvalError {
	UnhandledSyntax,
	IllegalAssignment,
	InvalidTypeForImport,
	NotType,
	NotCallable,
	NotFound(String),
	ArgMismatch,
	MethodNotImplemented,
	UserCodeError(String),
}

pub type EvalResult = Result<Value, EvalError>;

pub fn assign_value(ctx: &EvalContext, name: &str, value: Value) -> EvalResult {
	ctx.env.borrow_mut().update(name, value.clone());
	Ok(value)
}

fn eval_assign(ctx: &EvalContext, dest: &Syntax, src: &Syntax) -> EvalResult {
	match dest {
		Syntax::Ident(name) => {
			let value = eval_in_env(ctx, src)?;
			assign_value(ctx, name, value)
		}
		_ => Err(EvalError::IllegalAssignment),
	}
}

fn eval_unop(ctx: &EvalContext, target: &Syntax, op: UnaryOperator) -> EvalResult {
	stdlib::ops::UNOP_LOOKUP.with(|m| {
		if let Some(method) = m.get(&op) {
			method
				.borrow()
				.call(ctx, &[(&"a".to_string(), eval_in_env(ctx, target)?)], &[])
		} else {
			Err(EvalError::MethodNotImplemented)
		}
	})
}

fn eval_binop(ctx: &EvalContext, lhs: &Syntax, rhs: &Syntax, op: BinaryOperator) -> EvalResult {
	match op {
		BinaryOperator::LAssign => eval_assign(ctx, lhs, rhs),
		BinaryOperator::RAssign => eval_assign(ctx, rhs, lhs),
		BinaryOperator::Access => {
			let rhs_ident = if let Syntax::Ident(name) = rhs {
				name
			} else {
				return Err(EvalError::InvalidTypeForImport);
			};

			if let Value::Library(lib_env) = eval_in_env(ctx, lhs)? {
				if let Some(val) = lib_env.borrow().get(rhs_ident) {
					Ok(val)
				} else {
					Err(EvalError::NotFound(rhs_ident.clone()))
				}
			} else {
				Err(EvalError::InvalidTypeForImport)
			}
		}
		// TODO: pipes can be expressed as a syntax rewrite pass before eval
		BinaryOperator::Pipe => match rhs {
			Syntax::Call {
				target,
				positional_args,
				named_args,
			} => {
				let mut new_args = positional_args.clone();
				new_args.insert(0, lhs.clone());
				eval_in_env(
					ctx,
					&Syntax::Call {
						target: target.clone(),
						positional_args: new_args,
						named_args: named_args.clone(),
					},
				)
			}
			_ => Err(EvalError::UnhandledSyntax),
		},
		_ => stdlib::ops::BINOP_LOOKUP.with(|m| {
			if let Some(method) = m.get(&op) {
				method.borrow().call(
					ctx,
					&[
						(&"a".to_string(), eval_in_env(ctx, lhs)?),
						(&"b".to_string(), eval_in_env(ctx, rhs)?),
					],
					&[],
				)
			} else {
				Err(EvalError::MethodNotImplemented)
			}
		}),
	}
}

fn resolve_lib(starting_env: EnvironmentPtr, from: &[String]) -> Result<EnvironmentPtr, EvalError> {
	let mut current_env = starting_env;
	for name in from {
		let val = { current_env.borrow().get(name) };
		if let Some(lib_value) = val {
			if let Value::Library(lib_env) = lib_value {
				current_env = lib_env;
			} else {
				return Err(EvalError::InvalidTypeForImport);
			}
		} else {
			return Err(EvalError::NotFound(name.clone()));
		}
	}
	Ok(current_env)
}

fn eval_import(ctx: &EvalContext, from: &[String], import: &Import) -> EvalResult {
	let lib_env = resolve_lib(ctx.library_env.clone(), from)?;

	match import {
		Import::Named(names) => {
			for name in names {
				if let Some(val) = lib_env.borrow().get(name) {
					ctx.env.borrow_mut().update(name, val);
				} else {
					return Err(EvalError::NotFound(name.clone()));
				}
			}
		}
		Import::Wildcard => lib_env.borrow().copy_to(&mut ctx.env.borrow_mut()),
	}

	Ok(Value::Null(()))
}

#[derive(Default, Clone)]
pub struct EvalContext {
	pub env: EnvironmentPtr,
	pub library_env: EnvironmentPtr,
}

impl EvalContext {
	pub fn new() -> Self {
		let global_env_ptr = Environment::new("global");
		let library_env_ptr = Environment::new("libraries");

		let add_lib = |env_ptr: EnvironmentPtr, add_to_global| {
			let lib_val = Value::Library(env_ptr.clone());
			let env = env_ptr.borrow();
			library_env_ptr.borrow_mut().update(env.name(), lib_val);

			if add_to_global {
				env.copy_to(&mut global_env_ptr.borrow_mut());
			}
		};

		add_lib(stdlib::core::env(), true);
		add_lib(stdlib::ops::env(), false);
		add_lib(stdlib::data::env(), false);

		EvalContext {
			env: global_env_ptr,
			library_env: library_env_ptr,
		}
	}

	pub fn child(&self, env_ptr: EnvironmentPtr) -> EvalContext {
		EvalContext {
			library_env: self.library_env.clone(),
			env: env_ptr,
		}
	}
}

pub fn eval_in_env_multi(ctx: &EvalContext, exprs: &[Syntax]) -> EvalResult {
	let mut ret = Value::Null(());
	for expr in exprs {
		ret = eval_in_env(ctx, expr)?;
	}
	Ok(ret)
}

pub fn eval_in_env(ctx: &EvalContext, expr: &Syntax) -> EvalResult {
	match expr {
		Syntax::Int(val) => Ok(Value::Int(*val)),
		Syntax::Float(val) => Ok(Value::Float(*val)),
		Syntax::String(val) => Ok(Value::String(val.clone().into_boxed_str())),
		Syntax::Bool(val) => Ok(Value::Bool(*val)),
		Syntax::Null => Ok(Value::Null(())),
		Syntax::BinaryOp { lhs, rhs, op } => eval_binop(ctx, lhs, rhs, *op),
		Syntax::UnaryOp { target, op } => eval_unop(ctx, target, *op),
		Syntax::Interpolate(_) => Err(EvalError::UnhandledSyntax),
		Syntax::Function {
			name,
			params,
			return_type,
			body,
		} => eval_function(ctx, name, params, return_type, body),
		Syntax::Use { from, import } => eval_import(ctx, from, import),
		Syntax::Ident(name) => {
			if let Some(val) = ctx.env.borrow().get(name) {
				Ok(val)
			} else {
				Err(EvalError::NotFound(name.clone()))
			}
		}
		Syntax::Call {
			target,
			positional_args,
			named_args: _,
		} => match eval_in_env(ctx, target)? {
			Value::Builtin(builtin) => eval_callable(ctx, &*builtin, positional_args),
			Value::Function(func) => eval_callable(ctx, &*func, positional_args),
			Value::Method(method) => eval_callable(ctx, &*method.borrow(), positional_args),
			_ => Err(EvalError::NotCallable),
		},
	}
}

pub fn eval(ctx: &EvalContext, exprs: &[Syntax]) -> EvalResult {
	eval_in_env_multi(&ctx, exprs)
}
