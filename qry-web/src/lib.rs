use qry::lang::parse;
use qry::runtime::{eval_multi, EvalContext};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
	fn alert(s: &str);
}

#[wasm_bindgen]
pub fn run(src: &str) {
	let ctx = EvalContext::new_with_stdlib();
	let syntax = parse(src, "<web>").unwrap();
	let result = eval_multi(&ctx, &syntax).unwrap();
	alert(&format!("got value: {:?}", result));
}
