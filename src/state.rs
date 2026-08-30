use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Currency {
    USD,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LoanStatus {
    ActiveSafe,
    Liquidatable,
    Closed,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FinancialState {
    pub balances: HashMap<String, u64>,
    pub assets: HashMap<String, String>,
    pub receivables: HashMap<String, u64>,
    pub debts: HashMap<String, u64>,
    pub collaterals: HashMap<String, u64>,
    pub loan_status: HashMap<String, LoanStatus>,
    pub required_ratio: HashMap<String, f64>,
    pub asset_value: HashMap<String, u64>,
}

impl Default for FinancialState {
    fn default() -> Self {
        Self::new()
    }
}

impl FinancialState {
    pub fn new() -> Self {
        Self {
            balances: HashMap::new(),
            assets: HashMap::new(),
            receivables: HashMap::new(),
            debts: HashMap::new(),
            collaterals: HashMap::new(),
            loan_status: HashMap::new(),
            required_ratio: HashMap::new(),
            asset_value: HashMap::new(),
        }
    }

    pub fn set_balance(&mut self, account: &str, amount: u64) {
        self.balances.insert(account.to_string(), amount);
    }

    pub fn get_balance(&self, account: &str) -> u64 {
        *self.balances.get(account).unwrap_or(&0)
    }

    pub fn try_add_balance(&mut self, account: &str, amount: u64) -> Result<(), String> {
        let current = self.get_balance(account);
        let next = current
            .checked_add(amount)
            .ok_or_else(|| format!("Balance overflow for {}", account))?;
        self.set_balance(account, next);
        Ok(())
    }

    pub fn add_balance(&mut self, account: &str, amount: u64) {
        self.try_add_balance(account, amount)
            .expect("financial balance overflow");
    }

    pub fn sub_balance(&mut self, account: &str, amount: u64) -> Result<(), String> {
        let current = self.get_balance(account);
        if current < amount {
            return Err(format!(
                "Insufficient balance for {}: has {}, needs {}",
                account, current, amount
            ));
        }
        self.set_balance(account, current - amount);
        Ok(())
    }

    pub fn total_cash(&self) -> u64 {
        self.balances.values().sum()
    }

    pub fn total_asset_value(&self) -> u64 {
        self.asset_value.values().sum()
    }

    pub fn total_receivables(&self) -> u64 {
        self.receivables.values().sum()
    }

    pub fn total_debts(&self) -> u64 {
        self.debts.values().sum()
    }

    pub fn checked_net_value(&self) -> Result<u64, String> {
        let assets = self
            .total_cash()
            .checked_add(self.total_asset_value())
            .and_then(|v| v.checked_add(self.total_receivables()))
            .ok_or_else(|| "Net value overflow".to_string())?;
        assets
            .checked_sub(self.total_debts())
            .ok_or_else(|| "Net value cannot be negative".to_string())
    }

    pub fn net_value(&self) -> u64 {
        self.checked_net_value().unwrap_or(0)
    }

    pub fn collateral_ratio(&self, loan_id: &str) -> Option<f64> {
        let debt = self.debts.get(loan_id)?;
        if *debt == 0 {
            return None;
        }
        let collateral = self.collaterals.get(loan_id)?;
        Some(*collateral as f64 / *debt as f64)
    }

    pub fn update_loan_status(&mut self, loan_id: &str) -> Result<(), String> {
        let ratio = self.collateral_ratio(loan_id);
        let required = self.required_ratio.get(loan_id).copied().unwrap_or(1.5);
        match ratio {
            Some(r) if r >= required => {
                self.loan_status
                    .insert(loan_id.to_string(), LoanStatus::ActiveSafe);
                Ok(())
            }
            Some(_) => {
                self.loan_status
                    .insert(loan_id.to_string(), LoanStatus::Liquidatable);
                Ok(())
            }
            None => {
                self.loan_status
                    .insert(loan_id.to_string(), LoanStatus::Closed);
                Ok(())
            }
        }
    }

    pub fn check_invariants(&self) -> Result<(), String> {
        for (account, bal) in &self.balances {
            if *bal > u64::MAX / 2 {
                return Err(format!("Balance overflow for {}", account));
            }
        }
        for (loan_id, status) in &self.loan_status {
            match status {
                LoanStatus::Closed => {
                    if let Some(d) = self.debts.get(loan_id) {
                        if *d > 0 {
                            return Err(format!("Loan {} Closed but debt > 0", loan_id));
                        }
                    }
                }
                LoanStatus::ActiveSafe => {
                    let ratio = self
                        .collateral_ratio(loan_id)
                        .ok_or_else(|| format!("Loan {} ActiveSafe but no ratio", loan_id))?;
                    let req = self.required_ratio.get(loan_id).copied().unwrap_or(1.5);
                    if ratio < req {
                        return Err(format!(
                            "Loan {} ActiveSafe but ratio {} < required {}",
                            loan_id, ratio, req
                        ));
                    }
                }
                LoanStatus::Liquidatable => {
                    let ratio = self
                        .collateral_ratio(loan_id)
                        .ok_or_else(|| format!("Loan {} Liquidatable but no ratio", loan_id))?;
                    let req = self.required_ratio.get(loan_id).copied().unwrap_or(1.5);
                    if ratio >= req {
                        return Err(format!(
                            "Loan {} Liquidatable but ratio {} >= required {}",
                            loan_id, ratio, req
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}
