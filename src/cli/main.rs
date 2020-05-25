use qry::lang::parse;
use qry::runtime::{eval_multi, Callable, EvalContext};
use qry::stdlib::ops::RUNTIME_OPS;
use rustyline::error::ReadlineError;
use rustyline::Editor;

fn main() {
	let mut rl = Editor::<()>::new();
	let ctx = EvalContext::new();

	let to_string = RUNTIME_OPS.with(|o| o.to_string.clone());

	loop {
		match rl.readline("qry> ") {
			Ok(line) => match parse(&line) {
				Ok(syntax) => match eval_multi(&ctx, &syntax) {
					Ok(value) => {
						let value_str = to_string
							.borrow()
							.call(&ctx, &[(&"a".to_string(), &value)], &[]);

						match value_str {
							Ok(v) => println!("{}", v.as_string()),
							Err(err) => println!("error rendering value: {:?}", err),
						}
					}
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
