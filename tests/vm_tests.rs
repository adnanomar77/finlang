use finlang_core::{
    bytecode::{compile, execution_statement, verify_execution_statement, Vm},
    compiler::compile_source,
    state::FinancialState,
};

fn mint_run() -> (
    FinancialState,
    finlang_core::bytecode::ExecutionTrace,
    finlang_core::bytecode::Bytecode,
) {
    let compiled = compile_source("mint(alice, 25)").unwrap();
    let code = compile(&compiled.typed_ast);
    let mut vm = Vm::new();
    let mut state = FinancialState::new();
    vm.execute(&code, &mut state).unwrap();
    (state, vm.trace, code)
}

#[test]
fn same_inputs_produce_same_state_and_trace_root() {
    let (a, ta, code_a) = mint_run();
    let (b, tb, code_b) = mint_run();
    assert_eq!(a, b);
    assert_eq!(ta.root, tb.root);
    assert_eq!(code_a, code_b);
}

#[test]
fn execution_statement_is_verifiable() {
    let compiled = compile_source("mint(alice, 25)").unwrap();
    let code = compile(&compiled.typed_ast);
    let initial = FinancialState::new();
    let mut final_state = initial.clone();
    let mut vm = Vm::new();
    vm.execute(&code, &mut final_state).unwrap();
    let statement = execution_statement(&code, &initial, &final_state, &vm.trace);
    assert!(verify_execution_statement(
        &code,
        &initial,
        &final_state,
        &vm.trace,
        &statement
    ));
}

#[test]
fn failed_vm_execution_rolls_back_state() {
    let compiled = compile_source("mint(alice, 25)").unwrap();
    let mut code = compile(&compiled.typed_ast);
    code.ops.insert(
        code.ops.len() - 1,
        finlang_core::bytecode::OpCode::Transfer("alice".into(), "bob".into()),
    );
    let mut state = FinancialState::new();
    let before = state.clone();
    let mut vm = Vm::new();
    assert!(vm.execute(&code, &mut state).is_err());
    assert_eq!(state, before);
}
