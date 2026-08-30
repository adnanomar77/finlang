use finlang_core::{
    bytecode::{compile, Vm},
    compiler::compile_source,
    interpreter::Value,
    state::FinancialState,
};

fn run(source: &str) -> Result<Value, String> {
    let c = compile_source(source).map_err(|e| e.to_string())?;
    let code = compile(&c.typed_ast);
    let mut vm = Vm::new();
    let mut state = FinancialState::new();
    vm.execute(&code, &mut state).map_err(|e| e.to_string())
}

#[test]
fn arithmetic_precedence_is_correct() {
    assert_eq!(run("2 + 3 * 4"), Ok(Value::U64(14)));
}
#[test]
fn comparison_and_if_are_executable() {
    assert_eq!(run("if 3 >= 2 then 9 else 0"), Ok(Value::U64(9)));
}
#[test]
fn false_branch_is_executable() {
    assert_eq!(run("if false then 9 else 4"), Ok(Value::U64(4)));
}
#[test]
fn division_by_zero_is_rejected_at_runtime() {
    assert!(run("10 / 0").is_err());
}
#[test]
fn invalid_if_type_is_rejected() {
    assert!(compile_source("if 1 then 2 else 3").is_err());
}
