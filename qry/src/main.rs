use qry_lang::parse;
use qry_runtime::{eval_multi, EvalContext, Value};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::env;
use std::fs;

fn print_value(ctx: &EvalContext, value: Value) {
	print!("({})", value.runtime_type().name());

	if let Some(to_string_func) = ctx.methods.to_string.resolve(&[value.runtime_type()]) {
		match to_string_func.call(&ctx, &[value], &[]) {
			Ok(value_str) => print!(" {}", value_str.as_string()),
			Err(err) => print!(" error in to_string: {}", err),
		}
	}

	println!();
}

fn repl() {
	let mut rl = Editor::<()>::new();
	let ctx = EvalContext::new_with_stdlib();

	loop {
		match rl.readline("> ") {
			Ok(line) => match parse(&line, "<repl>") {
				Ok(syntax) => match eval_multi(&ctx, &syntax) {
					Ok(value) => print_value(&ctx, value),
					Err(err) => println!("{}", err),
				},
				Err(err) => println!("parser {}", err),
			},
			Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
			Err(err) => {
				println!("error {:?}", err);
				break;
			}
		}
	}
}

fn main() {
	let args = env::args().skip(1).collect::<Vec<_>>();
	if args.is_empty() {
		repl();
		return;
	}

	let script_contents = fs::read_to_string(&args[0]).unwrap();
	let ast = parse(&script_contents, &args[0]).unwrap();
	let ctx = EvalContext::new_with_stdlib();
	if let Err(ex) = eval_multi(&ctx, &ast) {
		println!("{}", ex);
	}
}
