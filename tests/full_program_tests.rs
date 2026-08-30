use finlang_core::ast::*;
use finlang_core::effects::Effects;
use finlang_core::interpreter::Interpreter;
use finlang_core::state::{FinancialState, LoanStatus};
use finlang_core::type_checker::{check_program_with_env, type_check};
use finlang_core::types::{Type, TypeEnv};

fn setup_state() -> FinancialState {
    let mut state = FinancialState::new();
    state.set_balance("pool", 10000);
    state.set_balance("alice", 1000);
    state.set_balance("bob", 500);
    state.set_balance("liquidation_pool", 0);
    state.set_balance("surplus_pool", 0);
    state
        .assets
        .insert("collateral1".to_string(), "alice".to_string());
    state.asset_value.insert("collateral1".to_string(), 150);
    state
}

#[test]
fn loan_price_update_liquidate_cycle() {
    let program = Expr::Let {
        name: "raw1".to_string(),
        value: Box::new(Expr::OracleRead {
            feed: SourceId::FeedA,
        }),
        body: Box::new(Expr::Let {
            name: "amount_verified".to_string(),
            value: Box::new(Expr::Validate {
                oracle: Box::new(Expr::Var {
                    name: "raw1".to_string(),
                }),
                policy: PolicyId::PriceBounds,
            }),
            body: Box::new(Expr::Let {
                name: "amount".to_string(),
                value: Box::new(Expr::ToAmount {
                    verified: Box::new(Expr::Var {
                        name: "amount_verified".to_string(),
                    }),
                }),
                body: Box::new(Expr::Let {
                    name: "raw2".to_string(),
                    value: Box::new(Expr::OracleRead {
                        feed: SourceId::FeedB,
                    }),
                    body: Box::new(Expr::Let {
                        name: "coll_verified".to_string(),
                        value: Box::new(Expr::Validate {
                            oracle: Box::new(Expr::Var {
                                name: "raw2".to_string(),
                            }),
                            policy: PolicyId::PriceBounds,
                        }),
                        body: Box::new(Expr::Let {
                            name: "coll".to_string(),
                            value: Box::new(Expr::ToAmount {
                                verified: Box::new(Expr::Var {
                                    name: "coll_verified".to_string(),
                                }),
                            }),
                            body: Box::new(Expr::Let {
                                name: "loan1".to_string(),
                                value: Box::new(Expr::CreateLoan {
                                    borrower: "alice".to_string(),
                                    lender_pool: "pool".to_string(),
                                    loan_id: "loan1".to_string(),
                                    amount: Box::new(Expr::Var {
                                        name: "amount".to_string(),
                                    }),
                                    collateral_asset: "collateral1".to_string(),
                                    collateral_value: Box::new(Expr::Var {
                                        name: "coll".to_string(),
                                    }),
                                    required_ratio: 1.5,
                                }),
                                body: Box::new(Expr::Let {
                                    name: "raw3".to_string(),
                                    value: Box::new(Expr::OracleRead {
                                        feed: SourceId::FeedA,
                                    }),
                                    body: Box::new(Expr::Let {
                                        name: "price_verified".to_string(),
                                        value: Box::new(Expr::Validate {
                                            oracle: Box::new(Expr::Var {
                                                name: "raw3".to_string(),
                                            }),
                                            policy: PolicyId::PriceBounds,
                                        }),
                                        body: Box::new(Expr::Let {
                                            name: "new_price".to_string(),
                                            value: Box::new(Expr::ToAmount {
                                                verified: Box::new(Expr::Var {
                                                    name: "price_verified".to_string(),
                                                }),
                                            }),
                                            body: Box::new(Expr::Let {
                                                name: "loan2".to_string(),
                                                value: Box::new(Expr::PriceUpdate {
                                                    loan: Box::new(Expr::Var {
                                                        name: "loan1".to_string(),
                                                    }),
                                                    new_price: Box::new(Expr::Var {
                                                        name: "new_price".to_string(),
                                                    }),
                                                }),
                                                body: Box::new(Expr::Liquidate {
                                                    loan: Box::new(Expr::Var {
                                                        name: "loan2".to_string(),
                                                    }),
                                                }),
                                            }),
                                        }),
                                    }),
                                }),
                            }),
                        }),
                    }),
                }),
            }),
        }),
    };

    let mut env = TypeEnv::new();
    let mut effects = Effects::new();
    let (typed_expr, ty) = type_check(&program, &mut env, &mut effects).unwrap();
    assert_eq!(ty, Type::Unit);
    assert!(env.linear.bindings.is_empty());

    let mut state = setup_state();
    let mut interpreter = Interpreter::new();
    interpreter.set_oracle_value("FeedA", 100);
    interpreter.set_oracle_value("FeedB", 150);
    interpreter.set_oracle_value("FeedA", 120);

    let result = interpreter.interpret(&typed_expr, &mut state);
    assert!(result.is_ok(), "Interpretation failed: {:?}", result);
    assert_eq!(state.loan_status.get("loan1").unwrap(), &LoanStatus::Closed);
}

#[test]
fn old_loan_is_consumed_after_update() {
    let program = Expr::Let {
        name: "raw1".to_string(),
        value: Box::new(Expr::OracleRead {
            feed: SourceId::FeedA,
        }),
        body: Box::new(Expr::Let {
            name: "amount_verified".to_string(),
            value: Box::new(Expr::Validate {
                oracle: Box::new(Expr::Var {
                    name: "raw1".to_string(),
                }),
                policy: PolicyId::PriceBounds,
            }),
            body: Box::new(Expr::Let {
                name: "amount".to_string(),
                value: Box::new(Expr::ToAmount {
                    verified: Box::new(Expr::Var {
                        name: "amount_verified".to_string(),
                    }),
                }),
                body: Box::new(Expr::Let {
                    name: "raw2".to_string(),
                    value: Box::new(Expr::OracleRead {
                        feed: SourceId::FeedB,
                    }),
                    body: Box::new(Expr::Let {
                        name: "coll_verified".to_string(),
                        value: Box::new(Expr::Validate {
                            oracle: Box::new(Expr::Var {
                                name: "raw2".to_string(),
                            }),
                            policy: PolicyId::PriceBounds,
                        }),
                        body: Box::new(Expr::Let {
                            name: "coll".to_string(),
                            value: Box::new(Expr::ToAmount {
                                verified: Box::new(Expr::Var {
                                    name: "coll_verified".to_string(),
                                }),
                            }),
                            body: Box::new(Expr::Let {
                                name: "loan1".to_string(),
                                value: Box::new(Expr::CreateLoan {
                                    borrower: "alice".to_string(),
                                    lender_pool: "pool".to_string(),
                                    loan_id: "loan1".to_string(),
                                    amount: Box::new(Expr::Var {
                                        name: "amount".to_string(),
                                    }),
                                    collateral_asset: "collateral1".to_string(),
                                    collateral_value: Box::new(Expr::Var {
                                        name: "coll".to_string(),
                                    }),
                                    required_ratio: 1.5,
                                }),
                                body: Box::new(Expr::Let {
                                    name: "raw3".to_string(),
                                    value: Box::new(Expr::OracleRead {
                                        feed: SourceId::FeedA,
                                    }),
                                    body: Box::new(Expr::Let {
                                        name: "price_verified".to_string(),
                                        value: Box::new(Expr::Validate {
                                            oracle: Box::new(Expr::Var {
                                                name: "raw3".to_string(),
                                            }),
                                            policy: PolicyId::PriceBounds,
                                        }),
                                        body: Box::new(Expr::Let {
                                            name: "price".to_string(),
                                            value: Box::new(Expr::ToAmount {
                                                verified: Box::new(Expr::Var {
                                                    name: "price_verified".to_string(),
                                                }),
                                            }),
                                            body: Box::new(Expr::PriceUpdate {
                                                loan: Box::new(Expr::Var {
                                                    name: "loan1".to_string(),
                                                }),
                                                new_price: Box::new(Expr::Var {
                                                    name: "price".to_string(),
                                                }),
                                            }),
                                        }),
                                    }),
                                }),
                            }),
                        }),
                    }),
                }),
            }),
        }),
    };

    let mut env = TypeEnv::new();
    let mut effects = Effects::new();
    let (_, _) = type_check(&program, &mut env, &mut effects).unwrap();
    assert!(env.linear.bindings.is_empty(), "loan1 should be consumed");
}

#[test]
fn check_program_with_env_rejects_unused_linear() {
    let mut env = TypeEnv::new();
    env.linear
        .insert(
            "asset".to_string(),
            Type::LinearAsset(AssetKind::Money(Currency::USD)),
        )
        .unwrap();

    let expr = Expr::Let {
        name: "unused".to_string(),
        value: Box::new(Expr::Var {
            name: "asset".to_string(),
        }),
        body: Box::new(Expr::Int(1)),
    };

    let mut effects = Effects::new();
    let _ = type_check(&expr, &mut env, &mut effects).unwrap();
    assert!(env.linear.bindings.contains_key("unused"));

    let mut env2 = TypeEnv::new();
    env2.linear
        .insert(
            "asset".to_string(),
            Type::LinearAsset(AssetKind::Money(Currency::USD)),
        )
        .unwrap();
    let mut effects2 = Effects::new();
    let res = check_program_with_env(&expr, &mut env2, &mut effects2);
    assert!(
        res.is_err(),
        "check_program_with_env should reject unused linear resources"
    );
}

#[test]
fn closed_loan_cannot_be_used() {
    // برنامج ينشئ قرضًا ثم يسدده بالكامل، ثم يحاول تحديث سعره
    let program = Expr::Let {
        name: "raw1".to_string(),
        value: Box::new(Expr::OracleRead {
            feed: SourceId::FeedA,
        }),
        body: Box::new(Expr::Let {
            name: "amount_verified".to_string(),
            value: Box::new(Expr::Validate {
                oracle: Box::new(Expr::Var {
                    name: "raw1".to_string(),
                }),
                policy: PolicyId::PriceBounds,
            }),
            body: Box::new(Expr::Let {
                name: "amount".to_string(),
                value: Box::new(Expr::ToAmount {
                    verified: Box::new(Expr::Var {
                        name: "amount_verified".to_string(),
                    }),
                }),
                body: Box::new(Expr::Let {
                    name: "raw2".to_string(),
                    value: Box::new(Expr::OracleRead {
                        feed: SourceId::FeedB,
                    }),
                    body: Box::new(Expr::Let {
                        name: "coll_verified".to_string(),
                        value: Box::new(Expr::Validate {
                            oracle: Box::new(Expr::Var {
                                name: "raw2".to_string(),
                            }),
                            policy: PolicyId::PriceBounds,
                        }),
                        body: Box::new(Expr::Let {
                            name: "coll".to_string(),
                            value: Box::new(Expr::ToAmount {
                                verified: Box::new(Expr::Var {
                                    name: "coll_verified".to_string(),
                                }),
                            }),
                            body: Box::new(Expr::Let {
                                name: "loan1".to_string(),
                                value: Box::new(Expr::CreateLoan {
                                    borrower: "alice".to_string(),
                                    lender_pool: "pool".to_string(),
                                    loan_id: "loan1".to_string(),
                                    amount: Box::new(Expr::Var {
                                        name: "amount".to_string(),
                                    }),
                                    collateral_asset: "collateral1".to_string(),
                                    collateral_value: Box::new(Expr::Var {
                                        name: "coll".to_string(),
                                    }),
                                    required_ratio: 1.5,
                                }),
                                body: Box::new(Expr::Let {
                                    name: "repay_result".to_string(),
                                    value: Box::new(Expr::Repay {
                                        borrower: "alice".to_string(),
                                        lender_pool: "pool".to_string(),
                                        loan: Box::new(Expr::Var {
                                            name: "loan1".to_string(),
                                        }),
                                        payment: Box::new(Expr::Int(100)),
                                    }),
                                    body: Box::new(Expr::Let {
                                        name: "raw3".to_string(),
                                        value: Box::new(Expr::OracleRead {
                                            feed: SourceId::FeedA,
                                        }),
                                        body: Box::new(Expr::Let {
                                            name: "price_verified".to_string(),
                                            value: Box::new(Expr::Validate {
                                                oracle: Box::new(Expr::Var {
                                                    name: "raw3".to_string(),
                                                }),
                                                policy: PolicyId::PriceBounds,
                                            }),
                                            body: Box::new(Expr::Let {
                                                name: "price".to_string(),
                                                value: Box::new(Expr::ToAmount {
                                                    verified: Box::new(Expr::Var {
                                                        name: "price_verified".to_string(),
                                                    }),
                                                }),
                                                body: Box::new(Expr::PriceUpdate {
                                                    loan: Box::new(Expr::Var {
                                                        name: "repay_result".to_string(),
                                                    }),
                                                    new_price: Box::new(Expr::Var {
                                                        name: "price".to_string(),
                                                    }),
                                                }),
                                            }),
                                        }),
                                    }),
                                }),
                            }),
                        }),
                    }),
                }),
            }),
        }),
    };

    let mut env = TypeEnv::new();
    let mut effects = Effects::new();
    let (typed_expr, _) = type_check(&program, &mut env, &mut effects).unwrap();

    let mut state = setup_state();
    let mut interpreter = Interpreter::new();
    interpreter.set_oracle_value("FeedA", 100);
    interpreter.set_oracle_value("FeedB", 150);
    interpreter.set_oracle_value("FeedA", 120);

    let result = interpreter.interpret(&typed_expr, &mut state);
    assert!(result.is_err(), "Expected error because loan is closed");
}

#[test]
fn transfer_moves_asset() {
    let program = Expr::Let {
        name: "asset".to_string(),
        value: Box::new(Expr::Var {
            name: "myasset".to_string(),
        }),
        body: Box::new(Expr::Transfer {
            from: "alice".to_string(),
            to: "bob".to_string(),
            asset: Box::new(Expr::Var {
                name: "asset".to_string(),
            }),
        }),
    };

    let mut env = TypeEnv::new();
    env.linear
        .insert(
            "myasset".to_string(),
            Type::LinearAsset(AssetKind::Money(Currency::USD)),
        )
        .unwrap();
    let mut effects = Effects::new();
    let (typed_expr, _) = type_check(&program, &mut env, &mut effects).unwrap();
    assert!(env.linear.bindings.is_empty());

    let mut state = setup_state();
    state
        .assets
        .insert("myasset".to_string(), "alice".to_string());
    state.asset_value.insert("myasset".to_string(), 100);

    let mut interpreter = Interpreter::new();
    interpreter.set_asset("myasset", "myasset");

    let result = interpreter.interpret(&typed_expr, &mut state);
    assert!(result.is_ok(), "Interpretation failed: {:?}", result);
    assert_eq!(state.assets.get("myasset").unwrap(), "bob");
}
