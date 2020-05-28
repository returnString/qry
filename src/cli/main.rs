use qry::lang::parse;
use qry::runtime::{eval_multi, Callable, EvalContext, Value};
use rustyline::error::ReadlineError;
use rustyline::Editor;

fn print_value(ctx: &EvalContext, value: Value) {
	print!("({})", value.runtime_type().name());

	if let Ok(value_str) = ctx.methods.to_string.call(&ctx, &[value], &[]) {
		print!(" {}", value_str.as_string());
	}

	println!();
}

fn main() {
	let mut rl = Editor::<()>::new();
	let ctx = EvalContext::new_with_stdlib();

	loop {
		match rl.readline("> ") {
			Ok(line) => match parse(&line) {
				Ok(syntax) => match eval_multi(&ctx, &syntax) {
					Ok(value) => print_value(&ctx, value),
					Err(err) => println!("runtime error: {:?}", err),
				},
				Err(err) => println!("parser error {:?}", err),
			},
			Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
			Err(err) => {
				println!("error {:?}", err);
				break;
			}
		}
	}
}
