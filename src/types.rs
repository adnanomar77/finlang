use crate::ast::{AssetKind, Currency, PolicyId, SourceId};
use std::collections::HashMap;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AmountKind {
    Plain,
    FromVerified { policy: PolicyId, source: SourceId },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Amount {
        currency: Currency,
        kind: AmountKind,
    },
    LinearAsset(AssetKind),
    Account,
    Oracle(Currency, SourceId),
    Verified(Currency, PolicyId, SourceId),
    Unit,
    Bool,
    Function {
        params: Vec<Type>,
        result: Box<Type>,
    },
    Loan {
        id: String,
    },
    Debt(String, Currency),
    Collateral(String, Currency),
}
#[derive(Debug, Clone, Default)]
pub struct LinearContext {
    pub bindings: HashMap<String, Type>,
}
impl LinearContext {
    pub fn new() -> Self {
        Self::default()
    }
    #[allow(clippy::map_entry)]
    pub fn insert(&mut self, name: String, ty: Type) -> Result<(), String> {
        if self.bindings.contains_key(&name) {
            Err(format!("Linear resource '{}' already defined", name))
        } else {
            self.bindings.insert(name, ty);
            Ok(())
        }
    }
    pub fn take(&mut self, name: &str) -> Result<Type, String> {
        self.bindings
            .remove(name)
            .ok_or_else(|| format!("Linear resource '{}' not found or already consumed", name))
    }
}
#[derive(Debug, Clone, Default)]
pub struct TypeEnv {
    pub vars: HashMap<String, Type>,
    pub linear: LinearContext,
    pub policies: HashMap<String, Type>,
}
impl TypeEnv {
    pub fn new() -> Self {
        Self::default()
    }
}
