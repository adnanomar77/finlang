use crate::state::{FinancialState, LoanStatus};

pub fn mint(state: &mut FinancialState, account: &str, amount: u64) -> Result<(), String> {
    state.try_add_balance(account, amount)?;
    state.check_invariants()?;
    Ok(())
}

pub fn transfer_asset(
    state: &mut FinancialState,
    asset_id: &str,
    from: &str,
    to: &str,
) -> Result<(), String> {
    let owner = state
        .assets
        .get(asset_id)
        .ok_or_else(|| format!("Asset {} not found", asset_id))?;
    if owner != from {
        return Err(format!("Asset {} not owned by {}", asset_id, from));
    }
    state.assets.insert(asset_id.to_string(), to.to_string());
    state.check_invariants()?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn create_loan(
    state: &mut FinancialState,
    borrower: &str,
    lender_pool: &str,
    loan_id: &str,
    amount: u64,
    collateral_asset_id: &str,
    collateral_value: u64,
    required_ratio: f64,
) -> Result<(), String> {
    if amount == 0 {
        return Err("Loan amount must be > 0".to_string());
    }
    if required_ratio <= 0.0 {
        return Err("Required ratio must be > 0".to_string());
    }
    if state.debts.contains_key(loan_id) || state.loan_status.contains_key(loan_id) {
        return Err(format!("Loan {} already exists", loan_id));
    }
    let owner = state
        .assets
        .get(collateral_asset_id)
        .ok_or_else(|| format!("Collateral asset {} not found", collateral_asset_id))?;
    if owner != borrower {
        return Err(format!(
            "Collateral asset {} not owned by borrower",
            collateral_asset_id
        ));
    }
    if collateral_value == 0 {
        return Err("Collateral value must be > 0".to_string());
    }
    if (collateral_value as f64 / amount as f64) < required_ratio {
        return Err(format!(
            "Collateral ratio too low: {} / {} < {}",
            collateral_value, amount, required_ratio
        ));
    }

    state.sub_balance(lender_pool, amount)?;
    state.try_add_balance(borrower, amount)?;

    state
        .assets
        .insert(collateral_asset_id.to_string(), format!("loan:{}", loan_id));
    state
        .asset_value
        .insert(collateral_asset_id.to_string(), collateral_value);

    state.receivables.insert(loan_id.to_string(), amount);
    state.debts.insert(loan_id.to_string(), amount);
    state
        .collaterals
        .insert(loan_id.to_string(), collateral_value);
    state
        .required_ratio
        .insert(loan_id.to_string(), required_ratio);

    state.update_loan_status(loan_id)?;
    state.check_invariants()?;
    Ok(())
}

pub fn repay(
    state: &mut FinancialState,
    borrower: &str,
    lender_pool: &str,
    loan_id: &str,
    payment: u64,
) -> Result<(), String> {
    let debt = *state
        .debts
        .get(loan_id)
        .ok_or_else(|| format!("Loan {} not found", loan_id))?;
    if payment == 0 {
        return Err("Payment must be > 0".to_string());
    }
    if payment > debt {
        return Err("Payment exceeds debt".to_string());
    }

    state.sub_balance(borrower, payment)?;
    state.try_add_balance(lender_pool, payment)?;

    let new_debt = debt - payment;
    state.debts.insert(loan_id.to_string(), new_debt);
    state.receivables.insert(loan_id.to_string(), new_debt);

    if new_debt == 0 {
        state
            .loan_status
            .insert(loan_id.to_string(), LoanStatus::Closed);
        if let Some(asset_id) = state.assets.iter().find_map(|(id, owner)| {
            if owner == &format!("loan:{}", loan_id) {
                Some(id.clone())
            } else {
                None
            }
        }) {
            state.assets.insert(asset_id.clone(), borrower.to_string());
        }
        state.debts.remove(loan_id);
        state.receivables.remove(loan_id);
    } else {
        state.update_loan_status(loan_id)?;
    }

    state.check_invariants()?;
    Ok(())
}

pub fn price_update(
    state: &mut FinancialState,
    loan_id: &str,
    new_collateral_value: u64,
) -> Result<(), String> {
    if new_collateral_value == 0 {
        return Err("Price update must be > 0".to_string());
    }
    if !state.debts.contains_key(loan_id) {
        return Err(format!("Loan {} not found", loan_id));
    }
    let debt = state.debts[loan_id];
    if debt == 0 {
        state
            .loan_status
            .insert(loan_id.to_string(), LoanStatus::Closed);
    } else {
        state
            .collaterals
            .insert(loan_id.to_string(), new_collateral_value);
        if let Some(asset_id) = state.assets.iter().find_map(|(id, owner)| {
            if owner == &format!("loan:{}", loan_id) {
                Some(id.clone())
            } else {
                None
            }
        }) {
            state.asset_value.insert(asset_id, new_collateral_value);
        }
        state.update_loan_status(loan_id)?;
    }
    state.check_invariants()?;
    Ok(())
}

pub fn liquidate(state: &mut FinancialState, loan_id: &str) -> Result<(), String> {
    if state.loan_status.get(loan_id) != Some(&LoanStatus::Liquidatable) {
        return Err(format!("Loan {} is not Liquidatable", loan_id));
    }

    let debt = *state
        .debts
        .get(loan_id)
        .ok_or_else(|| format!("Loan {} not found", loan_id))?;
    if debt == 0 {
        return Err("Loan already closed".to_string());
    }

    let collateral_value = *state
        .collaterals
        .get(loan_id)
        .ok_or_else(|| format!("Loan {} has no collateral", loan_id))?;

    if collateral_value < debt {
        return Err("Collateral value insufficient to cover debt".to_string());
    }

    state.try_add_balance("liquidation_pool", debt)?;
    let surplus = collateral_value - debt;
    if surplus > 0 {
        state.try_add_balance("surplus_pool", surplus)?;
    }

    state.debts.insert(loan_id.to_string(), 0);
    state.receivables.insert(loan_id.to_string(), 0);
    state
        .loan_status
        .insert(loan_id.to_string(), LoanStatus::Closed);

    if let Some(asset_id) = state.assets.iter().find_map(|(id, owner)| {
        if owner == &format!("loan:{}", loan_id) {
            Some(id.clone())
        } else {
            None
        }
    }) {
        state.assets.remove(&asset_id);
        state.asset_value.remove(&asset_id);
    }

    state.check_invariants()?;
    Ok(())
}
