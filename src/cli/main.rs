use qry::lang::parse;
use qry::runtime::{eval, InterpreterState};
use rustyline::error::ReadlineError;
use rustyline::Editor;

fn main() {
	let mut rl = Editor::<()>::new();
	let mut state = InterpreterState::new();

	loop {
		match rl.readline("qry> ") {
			Ok(line) => match parse(&line) {
				Ok(syntax) => match eval(&mut state, &syntax) {
					Ok(value) => println!("{:?}", value),
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
