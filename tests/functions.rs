use finlang_core::{
    bytecode::{compile, Vm},
    compiler::compile_source,
    interpreter::Value,
    state::FinancialState,
};
fn run(s: &str) -> Result<Value, String> {
    let c = compile_source(s).map_err(|e| e.to_string())?;
    let code = compile(&c.typed_ast);
    let mut vm = Vm::new();
    let mut st = FinancialState::new();
    vm.execute(&code, &mut st).map_err(|e| e.to_string())
}
#[test]
fn function_definition_and_call() {
    assert_eq!(
        run("let add = fn(x: Amount, y: Amount) -> Amount { x + y } in add(100, 50)"),
        Ok(Value::U64(150))
    );
}
#[test]
fn nested_calls_are_deterministic() {
    let s = "let add = fn(x: Amount, y: Amount) -> Amount { x + y } in add(add(1, 2), 3)";
    assert_eq!(run(s), Ok(Value::U64(6)));
    assert_eq!(run(s), run(s));
}
#[test]
fn invalid_function_argument_count_is_rejected() {
    assert!(
        compile_source("let add = fn(x: Amount, y: Amount) -> Amount { x + y } in add(1)").is_err()
    );
}
#[test]
fn invalid_function_argument_type_is_rejected() {
    assert!(compile_source("let f = fn(x: Amount) -> Amount { x } in f(true)").is_err());
}
