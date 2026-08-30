#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Currency {
    USD,
    EUR,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SourceId {
    FeedA,
    FeedB,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PolicyId {
    PriceBounds,
    Named(String),
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AssetKind {
    Money(Currency),
}
pub type AccountId = String;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Int(u64),
    Bool(bool),
    Function {
        params: Vec<String>,
        body: Box<Expr>,
    },
    Call {
        function: Box<Expr>,
        args: Vec<Expr>,
    },
    PolicyDef {
        name: String,
        parameter: String,
        predicate: Box<Expr>,
        body: Box<Expr>,
    },
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    If {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
    Transfer {
        from: AccountId,
        to: AccountId,
        asset: Box<Expr>,
    },
    Mint {
        account: AccountId,
        amount: Box<Expr>,
    },
    OracleRead {
        feed: SourceId,
    },
    Validate {
        oracle: Box<Expr>,
        policy: PolicyId,
    },
    ToAmount {
        verified: Box<Expr>,
    },
    UnsafeAssumeTrusted {
        oracle: Box<Expr>,
    },
    Let {
        name: String,
        value: Box<Expr>,
        body: Box<Expr>,
    },
    Var {
        name: String,
    },
    CreateLoan {
        borrower: AccountId,
        lender_pool: AccountId,
        loan_id: String,
        amount: Box<Expr>,
        collateral_asset: String,
        collateral_value: Box<Expr>,
        required_ratio: f64,
    },
    Repay {
        borrower: AccountId,
        lender_pool: AccountId,
        loan: Box<Expr>,
        payment: Box<Expr>,
    },
    PriceUpdate {
        loan: Box<Expr>,
        new_price: Box<Expr>,
    },
    Liquidate {
        loan: Box<Expr>,
    },
}
