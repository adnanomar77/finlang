use finlang_core::{
    bytecode::{compile, Vm},
    compiler::compile_source,
    interpreter::Value,
    state::FinancialState,
};
fn run(s: &str, oracle: u64) -> Result<Value, String> {
    let c = compile_source(s).map_err(|e| e.to_string())?;
    let code = compile(&c.typed_ast);
    let mut vm = Vm::new();
    vm.set_oracle_value(finlang_core::ast::SourceId::FeedA, oracle);
    let mut st = FinancialState::new();
    vm.execute(&code, &mut st).map_err(|e| e.to_string())
}
#[test]
fn policy_true_passes() {
    assert!(run(
        "policy Minimum(x: Amount) { x >= 100 } in validate(oracleRead(feedA), Minimum)",
        150
    )
    .is_ok());
}
#[test]
fn policy_false_rejects() {
    assert!(run(
        "policy Minimum(x: Amount) { x >= 100 } in validate(oracleRead(feedA), Minimum)",
        50
    )
    .is_err());
}
#[test]
fn undefined_policy_rejects() {
    assert!(compile_source("validate(oracleRead(feedA), Missing)").is_err());
}
#[test]
fn non_boolean_policy_rejects() {
    assert!(compile_source("policy Bad(x: Amount) { x + 1 } in 1").is_err());
}
#[test]
fn policy_is_deterministic() {
    let s = "policy Minimum(x: Amount) { x >= 100 } in validate(oracleRead(feedA), Minimum)";
    assert_eq!(run(s, 150), run(s, 150));
}
