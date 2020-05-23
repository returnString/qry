use super::{eval_callable, eval_function, Callable, Environment, EnvironmentPtr, Value};
use crate::lang::syntax::*;
use crate::stdlib;

#[derive(Default)]
pub struct InterpreterState {
	global_env: EnvironmentPtr,
	library_env: EnvironmentPtr,
}

impl InterpreterState {
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

		add_lib(stdlib::core::types_module(), true);
		add_lib(stdlib::core::ops_module(), true);

		InterpreterState {
			global_env: global_env_ptr,
			library_env: library_env_ptr,
		}
	}
}

#[derive(Debug, PartialEq, Eq)]
pub enum InterpreterError {
	UnhandledSyntax,
	IllegalAssignment,
	InvalidTypeForImport,
	NotType,
	NotCallable,
	NotFound(String),
	ArgMismatch,
	MethodNotImplemented,
}

pub fn assign_value(
	ctx: &EvalContext,
	name: &str,
	value: Value,
) -> Result<Value, InterpreterError> {
	ctx.env.borrow_mut().update(name, value.clone());
	Ok(value)
}

fn eval_assign(ctx: &EvalContext, dest: &Syntax, src: &Syntax) -> Result<Value, InterpreterError> {
	match dest {
		Syntax::Ident(name) => {
			let value = eval_in_env(ctx, src)?;
			assign_value(ctx, name, value)
		}
		_ => Err(InterpreterError::IllegalAssignment),
	}
}

fn eval_binop(
	ctx: &EvalContext,
	lhs: &Syntax,
	rhs: &Syntax,
	op: BinaryOperator,
) -> Result<Value, InterpreterError> {
	match op {
		BinaryOperator::LAssign => eval_assign(ctx, lhs, rhs),
		BinaryOperator::RAssign => eval_assign(ctx, rhs, lhs),
		BinaryOperator::Access => {
			let rhs_ident = if let Syntax::Ident(name) = rhs {
				name
			} else {
				return Err(InterpreterError::InvalidTypeForImport);
			};

			if let Value::Library(lib_env) = eval_in_env(ctx, lhs)? {
				if let Some(val) = lib_env.borrow().get(rhs_ident) {
					Ok(val)
				} else {
					Err(InterpreterError::NotFound(rhs_ident.clone()))
				}
			} else {
				Err(InterpreterError::InvalidTypeForImport)
			}
		}
		_ => stdlib::core::BINOP_LOOKUP.with(|m| {
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
				Err(InterpreterError::MethodNotImplemented)
			}
		}),
	}
}

fn resolve_lib(
	starting_env: EnvironmentPtr,
	from: &[String],
) -> Result<EnvironmentPtr, InterpreterError> {
	let mut current_env = starting_env;
	for name in from {
		let val = { current_env.borrow().get(name) };
		if let Some(lib_value) = val {
			if let Value::Library(lib_env) = lib_value {
				current_env = lib_env;
			} else {
				return Err(InterpreterError::InvalidTypeForImport);
			}
		} else {
			return Err(InterpreterError::NotFound(name.clone()));
		}
	}
	Ok(current_env)
}

fn eval_import(
	ctx: &EvalContext,
	from: &[String],
	import: &Import,
) -> Result<Value, InterpreterError> {
	let lib_env = resolve_lib(ctx.library_env.clone(), from)?;

	match import {
		Import::Named(names) => {
			for name in names {
				if let Some(val) = lib_env.borrow().get(name) {
					ctx.env.borrow_mut().update(name, val);
				} else {
					return Err(InterpreterError::NotFound(name.clone()));
				}
			}
		}
		Import::Wildcard => lib_env.borrow().copy_to(&mut ctx.env.borrow_mut()),
	}

	Ok(Value::Null)
}

pub struct EvalContext {
	pub env: EnvironmentPtr,
	pub library_env: EnvironmentPtr,
}

impl EvalContext {
	pub fn child(&self, env_ptr: EnvironmentPtr) -> EvalContext {
		EvalContext {
			library_env: self.library_env.clone(),
			env: env_ptr,
		}
	}
}

pub fn eval_in_env_multi(ctx: &EvalContext, exprs: &[Syntax]) -> Result<Value, InterpreterError> {
	let mut ret = Value::Null;
	for expr in exprs {
		ret = eval_in_env(ctx, expr)?;
	}
	Ok(ret)
}

pub fn eval_in_env(ctx: &EvalContext, expr: &Syntax) -> Result<Value, InterpreterError> {
	match expr {
		Syntax::Int(val) => Ok(Value::Int(*val)),
		Syntax::Float(val) => Ok(Value::Float(*val)),
		Syntax::String(val) => Ok(Value::String(val.clone().into_boxed_str())),
		Syntax::Bool(val) => Ok(Value::Bool(*val)),
		Syntax::Null => Ok(Value::Null),
		Syntax::BinaryOp { lhs, rhs, op } => eval_binop(ctx, lhs, rhs, *op),
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
				Err(InterpreterError::NotFound(name.clone()))
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
			_ => Err(InterpreterError::NotCallable),
		},
		_ => Err(InterpreterError::UnhandledSyntax),
	}
}

pub fn eval(state: &mut InterpreterState, exprs: &[Syntax]) -> Result<Value, InterpreterError> {
	let ctx = EvalContext {
		env: state.global_env.clone(),
		library_env: state.library_env.clone(),
	};

	eval_in_env_multi(&ctx, exprs)
}
