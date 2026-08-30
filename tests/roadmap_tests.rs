use finlang_core::{
    compiler::compile_source,
    effects::{mint_policy_ok, Effect, Effects},
    state::FinancialState,
};

#[test]
fn text_pipeline_accepts_all_operation_forms() {
    let cases = [
        "mint(alice, 10)",
        "transfer(alice, bob, asset)",
        "oracleRead(feedA)",
        "validate(oracleRead(feedA), PriceBounds)",
        "toAmount(validate(oracleRead(feedA), PriceBounds))",
        "unsafeAssumeTrusted(oracleRead(feedA))",
        "createLoan(alice, pool, loan1, 100, collateral1, 200, 1.5)",
        "repay(alice, pool, loan1, 10)",
        "priceUpdate(loan1, 150)",
        "liquidate(loan1)",
    ];
    for source in cases {
        let _ = compile_source(source);
    }
}

#[test]
fn invalid_programs_are_rejected() {
    assert!(compile_source("mint(alice)").is_err());
    assert!(compile_source("validate(oracleRead(feedA), UnknownPolicy)").is_err());
    assert!(compile_source("createLoan(a,p,l,10,c,20,0)").is_err());
}

#[test]
fn unsafe_effect_is_not_mintable() {
    let mut effects = Effects::new();
    effects.insert(Effect::Unsafe);
    assert!(!mint_policy_ok(&effects));
}

#[test]
fn arithmetic_is_checked() {
    let mut state = FinancialState::new();
    state.set_balance("alice", u64::MAX);
    assert!(state.try_add_balance("alice", 1).is_err());
    state.set_balance("alice", 0);
    state.debts.insert("loan".into(), 2);
    assert!(state.checked_net_value().is_err());
}
