use qry::lang::parse;
use qry::runtime::{eval, Callable, InterpreterState, Value};
use qry::stdlib::ops::RUNTIME_OPS;
use rustyline::error::ReadlineError;
use rustyline::Editor;

fn main() {
	let mut rl = Editor::<()>::new();
	let mut state = InterpreterState::new();

	let to_string = RUNTIME_OPS.with(|o| o.to_string.clone());

	loop {
		match rl.readline("qry> ") {
			Ok(line) => match parse(&line) {
				Ok(syntax) => match eval(&mut state, &syntax) {
					Ok(value) => {
						let value_str = to_string.borrow().call(
							&state.root_eval_context(),
							&[(&"a".to_string(), value)],
							&[],
						);

						match value_str {
							Ok(Value::String(s)) => println!("{}", s),
							Err(err) => println!("error rendering value: {:?}", err),
							_ => unreachable!(),
						}
					}
					Err(err) => println!("interpreter error: {:?}", err),
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
