use crate::abstract_interpreter;
use crate::ast::{PolicyId, SourceId};
use crate::state::FinancialState;
use crate::typed_ast::TypedExpr;
use crate::types::Type;
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    U64(u64),
    Bool(bool),
    Function {
        params: Vec<String>,
        body: Box<TypedExpr>,
    },
    Loan(String),
    Asset(String),
    Unit,
}

#[derive(Clone)]
pub struct Interpreter {
    env: HashMap<String, Value>,
    oracle_values: HashMap<String, VecDeque<u64>>,
    policies: HashMap<String, (String, Box<TypedExpr>)>,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: HashMap::new(),
            oracle_values: HashMap::new(),
            policies: HashMap::new(),
        }
    }

    pub fn set_oracle_value(&mut self, feed: &str, value: u64) {
        self.oracle_values
            .entry(feed.to_string())
            .or_default()
            .push_back(value);
    }

    pub fn set_asset(&mut self, name: &str, asset_id: &str) {
        self.env
            .insert(name.to_string(), Value::Asset(asset_id.to_string()));
    }

    pub fn set_loan(&mut self, name: &str, loan_id: &str) {
        self.env
            .insert(name.to_string(), Value::Loan(loan_id.to_string()));
    }

    fn is_linear_type(ty: &Type) -> bool {
        matches!(
            ty,
            Type::Loan { .. } | Type::LinearAsset(_) | Type::Debt(..) | Type::Collateral(..)
        )
    }

    fn get_var(&mut self, name: &str, ty: &Type) -> Result<Value, String> {
        if Self::is_linear_type(ty) {
            self.env
                .remove(name)
                .ok_or_else(|| format!("Linear variable {} not found or already consumed", name))
        } else {
            self.env
                .get(name)
                .cloned()
                .ok_or_else(|| format!("Variable {} not found", name))
        }
    }

    pub fn interpret_atomic(
        &self,
        expr: &TypedExpr,
        state: &mut FinancialState,
    ) -> Result<Value, String> {
        let mut staged = self.clone();
        let mut staged_state = state.clone();
        let result = staged.interpret(expr, &mut staged_state);
        if result.is_ok() {
            *state = staged_state;
        }
        result
    }

    pub fn interpret(
        &mut self,
        expr: &TypedExpr,
        state: &mut FinancialState,
    ) -> Result<Value, String> {
        match expr {
            TypedExpr::Int { value, .. } => Ok(Value::U64(*value)),
            TypedExpr::Bool { value, .. } => Ok(Value::Bool(*value)),
            TypedExpr::Binary {
                op, left, right, ..
            } => {
                let l = self.interpret(left, state)?;
                let r = self.interpret(right, state)?;
                let result = match op {
                    crate::ast::BinOp::Add => Value::U64(match (l, r) {
                        (Value::U64(a), Value::U64(b)) => {
                            a.checked_add(b).ok_or("addition overflow")?
                        }
                        _ => return Err("arithmetic expects numbers".into()),
                    }),
                    crate::ast::BinOp::Sub => Value::U64(match (l, r) {
                        (Value::U64(a), Value::U64(b)) => {
                            a.checked_sub(b).ok_or("subtraction underflow")?
                        }
                        _ => return Err("arithmetic expects numbers".into()),
                    }),
                    crate::ast::BinOp::Mul => Value::U64(match (l, r) {
                        (Value::U64(a), Value::U64(b)) => {
                            a.checked_mul(b).ok_or("multiplication overflow")?
                        }
                        _ => return Err("arithmetic expects numbers".into()),
                    }),
                    crate::ast::BinOp::Div => Value::U64(match (l, r) {
                        (Value::U64(_), Value::U64(0)) => return Err("division by zero".into()),
                        (Value::U64(a), Value::U64(b)) => a / b,
                        _ => return Err("arithmetic expects numbers".into()),
                    }),
                    crate::ast::BinOp::Lt => Value::Bool(match (l, r) {
                        (Value::U64(a), Value::U64(b)) => a < b,
                        _ => return Err("comparison expects numbers".into()),
                    }),
                    crate::ast::BinOp::Gt => Value::Bool(match (l, r) {
                        (Value::U64(a), Value::U64(b)) => a > b,
                        _ => return Err("comparison expects numbers".into()),
                    }),
                    crate::ast::BinOp::Le => Value::Bool(match (l, r) {
                        (Value::U64(a), Value::U64(b)) => a <= b,
                        _ => return Err("comparison expects numbers".into()),
                    }),
                    crate::ast::BinOp::Ge => Value::Bool(match (l, r) {
                        (Value::U64(a), Value::U64(b)) => a >= b,
                        _ => return Err("comparison expects numbers".into()),
                    }),
                    crate::ast::BinOp::Eq => Value::Bool(l == r),
                };
                Ok(result)
            }
            TypedExpr::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => match self.interpret(condition, state)? {
                Value::Bool(true) => self.interpret(then_branch, state),
                Value::Bool(false) => self.interpret(else_branch, state),
                _ => Err("if condition must be boolean".into()),
            },

            TypedExpr::Function { .. } => Err("function runtime integration pending".into()),
            TypedExpr::Call { .. } => Err("function call runtime integration pending".into()),
            TypedExpr::PolicyDef {
                name,
                parameter,
                predicate,
                body,
                ..
            } => {
                self.policies
                    .insert(name.clone(), (parameter.clone(), predicate.clone()));
                self.interpret(body, state)
            }
            TypedExpr::Var { name, ty } => self.get_var(name, ty),

            TypedExpr::Let {
                name, value, body, ..
            } => {
                let val = self.interpret(value, state)?;
                self.env.insert(name.clone(), val);
                self.interpret(body, state)
            }

            TypedExpr::Transfer {
                from, to, asset, ..
            } => {
                let asset_val = self.interpret(asset, state)?;
                let asset_id = match asset_val {
                    Value::Asset(id) => id,
                    _ => return Err("transfer expects asset".to_string()),
                };
                abstract_interpreter::transfer_asset(state, &asset_id, from, to)?;
                Ok(Value::Unit)
            }

            TypedExpr::Mint {
                account, amount, ..
            } => {
                let amount_val = self.interpret(amount, state)?;
                let amt = match amount_val {
                    Value::U64(n) => n,
                    _ => return Err("mint expects numeric amount".to_string()),
                };
                abstract_interpreter::mint(state, account, amt)?;
                Ok(Value::Unit)
            }

            TypedExpr::OracleRead { feed, .. } => {
                let key = match feed {
                    SourceId::FeedA => "FeedA",
                    SourceId::FeedB => "FeedB",
                };
                let queue = self
                    .oracle_values
                    .get_mut(key)
                    .ok_or_else(|| format!("No oracle value for {}", key))?;
                let val = queue
                    .pop_front()
                    .ok_or_else(|| format!("No oracle value available for {}", key))?;
                Ok(Value::U64(val))
            }

            TypedExpr::Validate { oracle, policy, .. } => {
                let v = self.interpret(oracle, state)?;
                let value = match v {
                    Value::U64(n) => n,
                    _ => return Err("validate expects numeric value".to_string()),
                };
                match policy {
                    PolicyId::PriceBounds => {
                        if !(100..=200).contains(&value) {
                            return Err(format!("Validation failed for PriceBounds: {}", value));
                        }
                    }
                    PolicyId::Named(name) => {
                        let value = match self.interpret(oracle, state)? {
                            Value::U64(n) => n,
                            _ => return Err("user policy expects numeric value".into()),
                        };
                        let Some((parameter, predicate)) = self.policies.get(name).cloned() else {
                            return Err(format!("Policy '{}' is not defined", name));
                        };
                        let mut child = self.clone();
                        child.env.insert(parameter, Value::U64(value));
                        match child.interpret(&predicate, state)? {
                            Value::Bool(true) => {}
                            Value::Bool(false) => {
                                return Err(format!("policy '{}' rejected value {}", name, value))
                            }
                            _ => return Err("policy predicate must return Bool".into()),
                        }
                    }
                }
                Ok(Value::U64(value))
            }

            TypedExpr::ToAmount { verified, .. } => self.interpret(verified, state),

            TypedExpr::UnsafeAssumeTrusted { oracle, .. } => self.interpret(oracle, state),

            TypedExpr::CreateLoan {
                borrower,
                lender_pool,
                loan_id,
                amount,
                collateral_asset,
                collateral_value,
                required_ratio,
                ..
            } => {
                let amount_val = self.interpret(amount, state)?;
                let amt = match amount_val {
                    Value::U64(n) => n,
                    _ => return Err("createLoan expects numeric amount".to_string()),
                };
                let coll_val = self.interpret(collateral_value, state)?;
                let coll_u64 = match coll_val {
                    Value::U64(n) => n,
                    _ => return Err("createLoan expects numeric collateral value".to_string()),
                };
                abstract_interpreter::create_loan(
                    state,
                    borrower,
                    lender_pool,
                    loan_id,
                    amt,
                    collateral_asset,
                    coll_u64,
                    *required_ratio,
                )?;
                Ok(Value::Loan(loan_id.clone()))
            }

            TypedExpr::Repay {
                borrower,
                lender_pool,
                loan,
                payment,
                ..
            } => {
                let loan_val = self.interpret(loan, state)?;
                let loan_id = match loan_val {
                    Value::Loan(id) => id,
                    _ => return Err("repay expects loan".to_string()),
                };
                let payment_val = self.interpret(payment, state)?;
                let pmt = match payment_val {
                    Value::U64(n) => n,
                    _ => return Err("repay expects numeric payment".to_string()),
                };
                abstract_interpreter::repay(state, borrower, lender_pool, &loan_id, pmt)?;
                Ok(Value::Loan(loan_id))
            }

            TypedExpr::PriceUpdate {
                loan, new_price, ..
            } => {
                let loan_val = self.interpret(loan, state)?;
                let loan_id = match loan_val {
                    Value::Loan(id) => id,
                    _ => return Err("priceUpdate expects loan".to_string()),
                };
                let price_val = self.interpret(new_price, state)?;
                let price = match price_val {
                    Value::U64(n) => n,
                    _ => return Err("priceUpdate expects numeric price".to_string()),
                };
                abstract_interpreter::price_update(state, &loan_id, price)?;
                Ok(Value::Loan(loan_id))
            }

            TypedExpr::Liquidate { loan, .. } => {
                let loan_val = self.interpret(loan, state)?;
                let loan_id = match loan_val {
                    Value::Loan(id) => id,
                    _ => return Err("liquidate expects loan".to_string()),
                };
                abstract_interpreter::liquidate(state, &loan_id)?;
                Ok(Value::Unit)
            }
        }
    }
}
