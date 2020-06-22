use qry_lang::parse;
use qry_runtime::{eval_multi, EvalContext};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
	fn alert(s: &str);
}

#[wasm_bindgen]
pub struct Interpreter {
	ctx: EvalContext,
}

#[wasm_bindgen]
impl Interpreter {
	#[wasm_bindgen(constructor)]
	pub fn ctor() -> Self {
		console_error_panic_hook::set_once();

		Interpreter {
			ctx: EvalContext::new_with_stdlib(),
		}
	}

	#[wasm_bindgen]
	pub fn eval(&self, src: &str) {
		let syntax = parse(src, "<web>").unwrap();
		match eval_multi(&self.ctx, &syntax) {
			Ok(value) => alert(&format!("got value: {:?}", value)),
			Err(ex) => alert(&format!("{}", ex)),
		}
	}
}
