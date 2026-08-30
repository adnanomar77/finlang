use crate::ast::*;
use crate::effects::{
    liquidation_policy_ok, loan_policy_ok, mint_policy_ok, repay_policy_ok, Effect, Effects,
};
use crate::typed_ast::TypedExpr;
use crate::types::{AmountKind, Type, TypeEnv};

pub fn type_check(
    expr: &Expr,
    env: &mut TypeEnv,
    effects: &mut Effects,
) -> Result<(TypedExpr, Type), String> {
    match expr {
        Expr::Function { params, body } => {
            let mut local = env.clone();
            let mut param_types = Vec::new();
            for p in params {
                let t = Type::Amount {
                    currency: Currency::USD,
                    kind: AmountKind::Plain,
                };
                local.vars.insert(p.clone(), t.clone());
                param_types.push(t);
            }
            let (typed_body, result) = type_check(body, &mut local, effects)?;
            let ty = Type::Function {
                params: param_types,
                result: Box::new(result),
            };
            Ok((
                TypedExpr::Function {
                    params: params.clone(),
                    body: Box::new(typed_body),
                    ty: ty.clone(),
                },
                ty,
            ))
        }
        Expr::Call { function, args } => {
            let (tf, ft) = type_check(function, env, effects)?;
            let Type::Function { params, result } = ft else {
                return Err("call expects a function".into());
            };
            if params.len() != args.len() {
                return Err(format!("function expects {} arguments", params.len()));
            }
            let mut ta = Vec::new();
            for (arg, expected) in args.iter().zip(params.iter()) {
                let (x, t) = type_check(arg, env, effects)?;
                if &t != expected {
                    return Err(format!("argument type mismatch: {:?} != {:?}", t, expected));
                }
                ta.push(x);
            }
            let ty = *result;
            Ok((
                TypedExpr::Call {
                    function: Box::new(tf),
                    args: ta,
                    ty: ty.clone(),
                },
                ty,
            ))
        }
        Expr::PolicyDef {
            name,
            parameter,
            predicate,
            body,
        } => {
            let mut local = env.clone();
            local.vars.insert(
                parameter.clone(),
                Type::Amount {
                    currency: Currency::USD,
                    kind: AmountKind::Plain,
                },
            );
            let (pred, pty) = type_check(predicate, &mut local, effects)?;
            if pty != Type::Bool {
                return Err("policy predicate must be Bool".into());
            }
            env.policies.insert(name.clone(), Type::Bool);
            let (b, ty) = type_check(body, env, effects)?;
            let out = ty.clone();
            Ok((
                TypedExpr::PolicyDef {
                    name: name.clone(),
                    parameter: parameter.clone(),
                    predicate: Box::new(pred),
                    body: Box::new(b),
                    ty: out.clone(),
                },
                out,
            ))
        }
        Expr::Bool(value) => Ok((
            TypedExpr::Bool {
                value: *value,
                ty: Type::Bool,
            },
            Type::Bool,
        )),
        Expr::Binary { op, left, right } => {
            let (lt, lty) = type_check(left, env, effects)?;
            let (rt, rty) = type_check(right, env, effects)?;
            let comparison = matches!(
                op,
                BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge | BinOp::Eq
            );
            if comparison {
                if lty != rty {
                    return Err(format!(
                        "comparison requires equal operand types, got {:?} and {:?}",
                        lty, rty
                    ));
                }
                let ty = Type::Bool;
                Ok((
                    TypedExpr::Binary {
                        op: op.clone(),
                        left: Box::new(lt),
                        right: Box::new(rt),
                        ty: ty.clone(),
                    },
                    ty,
                ))
            } else {
                match (&lty, &rty) {
                    (Type::Amount { currency: lc, .. }, Type::Amount { currency: rc, .. })
                        if lc == rc => {}
                    _ => {
                        return Err(format!(
                            "arithmetic requires two Amount operands, got {:?} and {:?}",
                            lty, rty
                        ))
                    }
                }
                let ty = lty.clone();
                Ok((
                    TypedExpr::Binary {
                        op: op.clone(),
                        left: Box::new(lt),
                        right: Box::new(rt),
                        ty: ty.clone(),
                    },
                    ty,
                ))
            }
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => {
            let (ct, cty) = type_check(condition, env, effects)?;
            if cty != Type::Bool {
                return Err(format!("if condition must be Bool, got {:?}", cty));
            }
            let mut then_env = env.clone();
            let mut then_effects = effects.clone();
            let mut else_env = env.clone();
            let mut else_effects = effects.clone();
            let (tt, tty) = type_check(then_branch, &mut then_env, &mut then_effects)?;
            let (et, ety) = type_check(else_branch, &mut else_env, &mut else_effects)?;
            if tty != ety {
                return Err(format!(
                    "if branches must have equal types, got {:?} and {:?}",
                    tty, ety
                ));
            }
            if !then_env.linear.bindings.is_empty() || !else_env.linear.bindings.is_empty() {
                return Err("linear resources cannot escape conditional branches".into());
            }
            let ty = tty.clone();
            Ok((
                TypedExpr::If {
                    condition: Box::new(ct),
                    then_branch: Box::new(tt),
                    else_branch: Box::new(et),
                    ty: ty.clone(),
                },
                ty,
            ))
        }
        Expr::Int(n) => {
            let ty = Type::Amount {
                currency: Currency::USD,
                kind: AmountKind::Plain,
            };
            Ok((
                TypedExpr::Int {
                    value: *n,
                    ty: ty.clone(),
                },
                ty,
            ))
        }

        Expr::Mint { account, amount } => {
            let (amount_typed, amount_ty) = type_check(amount, env, effects)?;
            match amount_ty {
                Type::Amount { .. } => {
                    if !mint_policy_ok(effects) {
                        return Err(format!("MintPolicy violated: {:?}", effects));
                    }
                    let ty = Type::Unit;
                    Ok((
                        TypedExpr::Mint {
                            account: account.clone(),
                            amount: Box::new(amount_typed),
                            ty: ty.clone(),
                        },
                        ty,
                    ))
                }
                _ => Err(format!("mint expects Amount, got {:?}", amount_ty)),
            }
        }

        Expr::OracleRead { feed } => {
            let source = feed.clone();
            effects.insert(Effect::UntrustedData(source.clone()));
            let ty = Type::Oracle(Currency::USD, source);
            Ok((
                TypedExpr::OracleRead {
                    feed: feed.clone(),
                    ty: ty.clone(),
                },
                ty,
            ))
        }

        Expr::Validate { oracle, policy } => {
            let (oracle_typed, oracle_ty) = type_check(oracle, env, effects)?;
            match oracle_ty {
                Type::Oracle(currency, source) => {
                    if let PolicyId::Named(name) = policy {
                        if !env.policies.contains_key(name) {
                            return Err(format!("Policy '{}' is not defined", name));
                        }
                    }
                    let obligation = Effect::UntrustedData(source.clone());
                    if !effects.remove(&obligation) {
                        return Err(format!(
                            "UntrustedData({:?}) not present; cannot validate",
                            source
                        ));
                    }
                    effects.insert(Effect::Validation(policy.clone(), source.clone()));
                    let ty = Type::Verified(currency, policy.clone(), source);
                    Ok((
                        TypedExpr::Validate {
                            oracle: Box::new(oracle_typed),
                            policy: policy.clone(),
                            ty: ty.clone(),
                        },
                        ty,
                    ))
                }
                _ => Err(format!("validate expects Oracle, got {:?}", oracle_ty)),
            }
        }

        Expr::ToAmount { verified } => {
            let (verified_typed, verified_ty) = type_check(verified, env, effects)?;
            match verified_ty {
                Type::Verified(currency, policy, source) => {
                    effects.insert(Effect::ValidatedData(source.clone()));
                    let ty = Type::Amount {
                        currency,
                        kind: AmountKind::FromVerified { policy, source },
                    };
                    Ok((
                        TypedExpr::ToAmount {
                            verified: Box::new(verified_typed),
                            ty: ty.clone(),
                        },
                        ty,
                    ))
                }
                _ => Err(format!("toAmount expects Verified, got {:?}", verified_ty)),
            }
        }

        Expr::UnsafeAssumeTrusted { oracle } => {
            let (oracle_typed, oracle_ty) = type_check(oracle, env, effects)?;
            match oracle_ty {
                Type::Oracle(currency, source) => {
                    effects.insert(Effect::TrustAssumption(source.clone()));
                    effects.insert(Effect::Unsafe);
                    let ty = Type::Amount {
                        currency,
                        kind: AmountKind::Plain,
                    };
                    Ok((
                        TypedExpr::UnsafeAssumeTrusted {
                            oracle: Box::new(oracle_typed),
                            ty: ty.clone(),
                        },
                        ty,
                    ))
                }
                _ => Err(format!(
                    "unsafeAssumeTrusted expects Oracle, got {:?}",
                    oracle_ty
                )),
            }
        }

        Expr::Transfer { from, to, asset } => {
            let (asset_typed, asset_ty) = type_check(asset, env, effects)?;
            match asset_ty {
                Type::LinearAsset(_) => {
                    let ty = Type::Unit;
                    Ok((
                        TypedExpr::Transfer {
                            from: from.clone(),
                            to: to.clone(),
                            asset: Box::new(asset_typed),
                            ty: ty.clone(),
                        },
                        ty,
                    ))
                }
                _ => Err(format!("transfer expects LinearAsset, got {:?}", asset_ty)),
            }
        }

        Expr::Let { name, value, body } => {
            let (value_typed, value_ty) = type_check(value, env, effects)?;
            match value_ty {
                Type::Loan { .. }
                | Type::LinearAsset(_)
                | Type::Debt(..)
                | Type::Collateral(..) => {
                    env.linear.insert(name.clone(), value_ty.clone())?;
                }
                _ => {
                    env.vars.insert(name.clone(), value_ty.clone());
                }
            }
            let (body_typed, body_ty) = type_check(body, env, effects)?;
            Ok((
                TypedExpr::Let {
                    name: name.clone(),
                    value: Box::new(value_typed),
                    body: Box::new(body_typed),
                    ty: body_ty.clone(),
                },
                body_ty,
            ))
        }

        Expr::Var { name } => {
            let ty = if let Some(t) = env.vars.get(name) {
                t.clone()
            } else {
                env.linear.take(name)?
            };
            Ok((
                TypedExpr::Var {
                    name: name.clone(),
                    ty: ty.clone(),
                },
                ty,
            ))
        }

        Expr::CreateLoan {
            borrower,
            lender_pool,
            loan_id,
            amount,
            collateral_asset,
            collateral_value,
            required_ratio,
        } => {
            let (amount_typed, amount_ty) = type_check(amount, env, effects)?;
            match amount_ty {
                Type::Amount {
                    kind: AmountKind::FromVerified { .. },
                    ..
                } => {}
                _ => return Err("createLoan expects verified amount".to_string()),
            }
            let (collateral_typed, collateral_ty) = type_check(collateral_value, env, effects)?;
            match collateral_ty {
                Type::Amount {
                    kind: AmountKind::FromVerified { .. },
                    ..
                } => {}
                _ => return Err("createLoan requires verified collateral value".to_string()),
            }
            if !loan_policy_ok(effects) {
                return Err(format!("LoanPolicy violated: {:?}", effects));
            }
            let ty = Type::Loan {
                id: loan_id.clone(),
            };
            Ok((
                TypedExpr::CreateLoan {
                    borrower: borrower.clone(),
                    lender_pool: lender_pool.clone(),
                    loan_id: loan_id.clone(),
                    amount: Box::new(amount_typed),
                    collateral_asset: collateral_asset.clone(),
                    collateral_value: Box::new(collateral_typed),
                    required_ratio: *required_ratio,
                    ty: ty.clone(),
                },
                ty,
            ))
        }

        Expr::Repay {
            borrower,
            lender_pool,
            loan,
            payment,
        } => {
            let (loan_typed, loan_ty) = type_check(loan, env, effects)?;
            let loan_id = match loan_ty {
                Type::Loan { id } => id,
                _ => return Err("repay expects Loan".to_string()),
            };
            let (payment_typed, payment_ty) = type_check(payment, env, effects)?;
            match payment_ty {
                Type::Amount { .. } => {
                    if !repay_policy_ok(effects) {
                        return Err(format!("RepayPolicy violated: {:?}", effects));
                    }
                    let ty = Type::Loan { id: loan_id };
                    Ok((
                        TypedExpr::Repay {
                            borrower: borrower.clone(),
                            lender_pool: lender_pool.clone(),
                            loan: Box::new(loan_typed),
                            payment: Box::new(payment_typed),
                            ty: ty.clone(),
                        },
                        ty,
                    ))
                }
                _ => Err("repay expects Amount for payment".to_string()),
            }
        }

        Expr::PriceUpdate { loan, new_price } => {
            let (loan_typed, loan_ty) = type_check(loan, env, effects)?;
            let loan_id = match loan_ty {
                Type::Loan { id } => id,
                _ => return Err("priceUpdate expects Loan".to_string()),
            };
            let (price_typed, price_ty) = type_check(new_price, env, effects)?;
            match price_ty {
                Type::Amount {
                    kind: AmountKind::FromVerified { .. },
                    ..
                } => {
                    let ty = Type::Loan { id: loan_id };
                    Ok((
                        TypedExpr::PriceUpdate {
                            loan: Box::new(loan_typed),
                            new_price: Box::new(price_typed),
                            ty: ty.clone(),
                        },
                        ty,
                    ))
                }
                _ => Err("priceUpdate requires verified price".to_string()),
            }
        }

        Expr::Liquidate { loan } => {
            let (loan_typed, loan_ty) = type_check(loan, env, effects)?;
            match loan_ty {
                Type::Loan { .. } => {
                    if !liquidation_policy_ok(effects) {
                        return Err(format!("LiquidationPolicy violated: {:?}", effects));
                    }
                    let ty = Type::Unit;
                    Ok((
                        TypedExpr::Liquidate {
                            loan: Box::new(loan_typed),
                            ty: ty.clone(),
                        },
                        ty,
                    ))
                }
                _ => Err("liquidate expects Loan".to_string()),
            }
        }
    }
}

pub fn check_program_with_env(
    expr: &Expr,
    env: &mut TypeEnv,
    effects: &mut Effects,
) -> Result<(TypedExpr, Type), String> {
    let (typed_expr, ty) = type_check(expr, env, effects)?;
    if !env.linear.bindings.is_empty() {
        return Err(format!(
            "Unused linear resources at end of program: {:?}",
            env.linear.bindings.keys().collect::<Vec<_>>()
        ));
    }
    Ok((typed_expr, ty))
}

pub fn check_program(expr: &Expr) -> Result<(TypedExpr, Type), String> {
    let mut env = TypeEnv::new();
    let mut effects = Effects::new();
    check_program_with_env(expr, &mut env, &mut effects)
}
