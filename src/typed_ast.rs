use crate::{
    ast::{AccountId, BinOp, PolicyId, SourceId},
    types::Type,
};
#[derive(Debug, Clone, PartialEq)]
pub enum TypedExpr {
    Int {
        value: u64,
        ty: Type,
    },
    Bool {
        value: bool,
        ty: Type,
    },
    Function {
        params: Vec<String>,
        body: Box<TypedExpr>,
        ty: Type,
    },
    Call {
        function: Box<TypedExpr>,
        args: Vec<TypedExpr>,
        ty: Type,
    },
    PolicyDef {
        name: String,
        parameter: String,
        predicate: Box<TypedExpr>,
        body: Box<TypedExpr>,
        ty: Type,
    },
    Binary {
        op: BinOp,
        left: Box<TypedExpr>,
        right: Box<TypedExpr>,
        ty: Type,
    },
    If {
        condition: Box<TypedExpr>,
        then_branch: Box<TypedExpr>,
        else_branch: Box<TypedExpr>,
        ty: Type,
    },
    Transfer {
        from: AccountId,
        to: AccountId,
        asset: Box<TypedExpr>,
        ty: Type,
    },
    Mint {
        account: AccountId,
        amount: Box<TypedExpr>,
        ty: Type,
    },
    OracleRead {
        feed: SourceId,
        ty: Type,
    },
    Validate {
        oracle: Box<TypedExpr>,
        policy: PolicyId,
        ty: Type,
    },
    ToAmount {
        verified: Box<TypedExpr>,
        ty: Type,
    },
    UnsafeAssumeTrusted {
        oracle: Box<TypedExpr>,
        ty: Type,
    },
    Let {
        name: String,
        value: Box<TypedExpr>,
        body: Box<TypedExpr>,
        ty: Type,
    },
    Var {
        name: String,
        ty: Type,
    },
    CreateLoan {
        borrower: AccountId,
        lender_pool: AccountId,
        loan_id: String,
        amount: Box<TypedExpr>,
        collateral_asset: String,
        collateral_value: Box<TypedExpr>,
        required_ratio: f64,
        ty: Type,
    },
    Repay {
        borrower: AccountId,
        lender_pool: AccountId,
        loan: Box<TypedExpr>,
        payment: Box<TypedExpr>,
        ty: Type,
    },
    PriceUpdate {
        loan: Box<TypedExpr>,
        new_price: Box<TypedExpr>,
        ty: Type,
    },
    Liquidate {
        loan: Box<TypedExpr>,
        ty: Type,
    },
}
