use crate::ast::{PolicyId, SourceId};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Effect {
    UntrustedData(SourceId),
    Validation(PolicyId, SourceId),
    ValidatedData(SourceId),
    TrustAssumption(SourceId),
    Unsafe,
}

pub type Effects = HashSet<Effect>;

pub fn mint_policy_ok(effects: &Effects) -> bool {
    effects.iter().all(|e| {
        !matches!(
            e,
            Effect::Unsafe | Effect::TrustAssumption(_) | Effect::UntrustedData(_)
        )
    })
}

pub fn loan_policy_ok(effects: &Effects) -> bool {
    mint_policy_ok(effects)
}

pub fn repay_policy_ok(effects: &Effects) -> bool {
    mint_policy_ok(effects)
}

pub fn liquidation_policy_ok(effects: &Effects) -> bool {
    mint_policy_ok(effects)
}

pub fn unsafe_present(effects: &Effects) -> bool {
    effects
        .iter()
        .any(|e| matches!(e, Effect::Unsafe | Effect::TrustAssumption(_)))
}
