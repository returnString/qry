use qry::lang::parse;
use qry::runtime::{eval, EvalContext, EvalResult, InterpreterError, Value};

pub fn eval_src(src: &str) -> EvalResult {
	let expr = parse(src).unwrap_or_else(|err| panic!("parse failed ({}): {:?}", src, err));
	eval(&EvalContext::new(), &expr)
}

pub fn eval_expect_values(cases: &[(&str, Value)]) {
	for (src, expectation) in cases {
		let result = eval_src(src).unwrap_or_else(|err| panic!("eval failed ({}): {:?}", src, err));
		assert_eq!(result, *expectation, "src: {}", src);
	}
}

pub fn eval_expect_errors(cases: &[(&str, InterpreterError)]) {
	for (src, expectation) in cases {
		let result = eval_src(src).expect_err(&format!("eval succeded unexpectedly for {}", src));
		assert_eq!(result, *expectation, "src: {}", src);
	}
}
