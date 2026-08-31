use crate::{
    abstract_interpreter,
    ast::{PolicyId, SourceId},
    interpreter::Value,
    state::FinancialState,
    typed_ast::TypedExpr,
};
use std::collections::{HashMap, VecDeque};

pub const BYTECODE_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq)]
pub enum OpCode {
    Push(u64),
    PushBool(bool),
    Binary(crate::ast::BinOp),
    Select {
        then_ops: Vec<OpCode>,
        else_ops: Vec<OpCode>,
    },
    Trap(String),
    Function {
        params: Vec<String>,
        body: Box<TypedExpr>,
    },
    Call(usize),
    DefinePolicy {
        name: String,
        parameter: String,
        predicate: Box<TypedExpr>,
    },
    CallPolicy(String),
    Load(String),
    Store(String),
    Pop,
    OracleRead(SourceId),
    Validate(PolicyId),
    ToAmount,
    UnsafeAssumeTrusted,
    Mint(String),
    Transfer(String, String),
    CreateLoan {
        borrower: String,
        lender_pool: String,
        loan_id: String,
        collateral_asset: String,
        required_ratio: f64,
    },
    Repay {
        borrower: String,
        lender_pool: String,
    },
    PriceUpdate,
    Liquidate,
    Halt,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Bytecode {
    pub version: u16,
    pub ops: Vec<OpCode>,
}
#[derive(Debug, Clone, PartialEq)]
pub enum VerifyError {
    Empty,
    MissingHalt,
    StackUnderflow(usize),
    InvalidVersion(u16),
}
#[derive(Debug, Clone, PartialEq)]
pub enum VmError {
    Verification(VerifyError),
    Runtime { pc: usize, message: String },
    StackUnderflow(usize),
    InvalidValue(usize),
}
#[derive(Debug, Clone, PartialEq)]
pub struct TraceStep {
    pub step: usize,
    pub pc: usize,
    pub opcode: OpCode,
    pub pre_digest: u64,
    pub post_digest: u64,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionTrace {
    pub steps: Vec<TraceStep>,
    pub root: u64,
}

pub fn compile(typed: &TypedExpr) -> Bytecode {
    let mut ops = Vec::new();
    emit(typed, &mut ops);
    ops.push(OpCode::Halt);
    Bytecode {
        version: BYTECODE_VERSION,
        ops,
    }
}
fn emit(e: &TypedExpr, out: &mut Vec<OpCode>) {
    match e {
        TypedExpr::Int { value, .. } => out.push(OpCode::Push(*value)),
        TypedExpr::Bool { value, .. } => out.push(OpCode::PushBool(*value)),
        TypedExpr::Binary {
            op, left, right, ..
        } => {
            emit(left, out);
            emit(right, out);
            out.push(OpCode::Binary(op.clone()));
        }
        TypedExpr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            emit(condition, out);
            let mut t = Vec::new();
            let mut e = Vec::new();
            emit(then_branch, &mut t);
            emit(else_branch, &mut e);
            out.push(OpCode::Select {
                then_ops: t,
                else_ops: e,
            });
        }
        TypedExpr::Function { params, body, .. } => out.push(OpCode::Function {
            params: params.clone(),
            body: body.clone(),
        }),
        TypedExpr::Call { function, args, .. } => {
            emit(function, out);
            for arg in args {
                emit(arg, out);
            }
            out.push(OpCode::Call(args.len()));
        }
        TypedExpr::PolicyDef {
            name,
            parameter,
            predicate,
            body,
            ..
        } => {
            out.push(OpCode::DefinePolicy {
                name: name.clone(),
                parameter: parameter.clone(),
                predicate: predicate.clone(),
            });
            emit(body, out);
        }
        TypedExpr::Var { name, .. } => out.push(OpCode::Load(name.clone())),
        TypedExpr::Let {
            name, value, body, ..
        } => {
            emit(value, out);
            out.push(OpCode::Store(name.clone()));
            emit(body, out);
        }
        TypedExpr::OracleRead { feed, .. } => out.push(OpCode::OracleRead(feed.clone())),
        TypedExpr::Validate { oracle, policy, .. } => {
            emit(oracle, out);
            match policy {
                PolicyId::PriceBounds => out.push(OpCode::Validate(policy.clone())),
                PolicyId::Named(name) => out.push(OpCode::CallPolicy(name.clone())),
            }
        }
        TypedExpr::ToAmount { verified, .. } => {
            emit(verified, out);
            out.push(OpCode::ToAmount);
        }
        TypedExpr::UnsafeAssumeTrusted { oracle, .. } => {
            emit(oracle, out);
            out.push(OpCode::UnsafeAssumeTrusted);
        }
        TypedExpr::Mint {
            account, amount, ..
        } => {
            emit(amount, out);
            out.push(OpCode::Mint(account.clone()));
        }
        TypedExpr::Transfer {
            from, to, asset, ..
        } => {
            emit(asset, out);
            out.push(OpCode::Transfer(from.clone(), to.clone()));
        }
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
            emit(amount, out);
            emit(collateral_value, out);
            out.push(OpCode::CreateLoan {
                borrower: borrower.clone(),
                lender_pool: lender_pool.clone(),
                loan_id: loan_id.clone(),
                collateral_asset: collateral_asset.clone(),
                required_ratio: *required_ratio,
            });
        }
        TypedExpr::Repay {
            borrower,
            lender_pool,
            loan,
            payment,
            ..
        } => {
            emit(loan, out);
            emit(payment, out);
            out.push(OpCode::Repay {
                borrower: borrower.clone(),
                lender_pool: lender_pool.clone(),
            });
        }
        TypedExpr::PriceUpdate {
            loan, new_price, ..
        } => {
            emit(loan, out);
            emit(new_price, out);
            out.push(OpCode::PriceUpdate);
        }
        TypedExpr::Liquidate { loan, .. } => {
            emit(loan, out);
            out.push(OpCode::Liquidate);
        }
    }
}

pub fn verify(code: &Bytecode) -> Result<(), VerifyError> {
    if code.version != BYTECODE_VERSION {
        return Err(VerifyError::InvalidVersion(code.version));
    }
    if code.ops.is_empty() {
        return Err(VerifyError::Empty);
    }
    if !matches!(code.ops.last(), Some(OpCode::Halt)) {
        return Err(VerifyError::MissingHalt);
    }
    let mut depth: isize = 0;
    for (pc, op) in code.ops.iter().enumerate() {
        let delta = match op {
            OpCode::Push(_) | OpCode::PushBool(_) | OpCode::Load(_) | OpCode::OracleRead(_) => 1,
            OpCode::Store(_) | OpCode::Pop => -1,
            OpCode::Validate(_) | OpCode::ToAmount | OpCode::UnsafeAssumeTrusted => 0,
            OpCode::Binary(_) => -1,
            OpCode::Select { .. } | OpCode::Trap(_) => 0,
            OpCode::Function { .. } => 1,
            OpCode::Call(n) => -(*n as isize),
            OpCode::DefinePolicy { .. } => 0,
            OpCode::CallPolicy(_) => 0,
            OpCode::Mint(_) | OpCode::Transfer(_, _) => 0,
            OpCode::CreateLoan { .. } => -1,
            OpCode::Repay { .. } | OpCode::PriceUpdate => -1,
            OpCode::Liquidate => 0,
            OpCode::Halt => 0,
        };
        depth += delta;
        if depth < 0 {
            return Err(VerifyError::StackUnderflow(pc));
        }
    }
    Ok(())
}

pub struct Vm {
    vars: HashMap<String, Value>,
    oracle_values: HashMap<SourceId, VecDeque<u64>>,
    policies: HashMap<String, (String, Box<TypedExpr>)>,
    pub trace: ExecutionTrace,
}
impl Default for Vm {
    fn default() -> Self {
        Self::new()
    }
}
impl Vm {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            oracle_values: HashMap::new(),
            policies: HashMap::new(),
            trace: ExecutionTrace {
                steps: Vec::new(),
                root: 0,
            },
        }
    }
    pub fn set_oracle_value(&mut self, feed: SourceId, value: u64) {
        self.oracle_values.entry(feed).or_default().push_back(value);
    }
    pub fn execute(
        &mut self,
        code: &Bytecode,
        state: &mut FinancialState,
    ) -> Result<Value, VmError> {
        verify(code).map_err(VmError::Verification)?;
        let mut staged = state.clone();
        let mut stack: Vec<Value> = Vec::new();
        for (pc, op) in code.ops.iter().enumerate() {
            let pre = digest_state(&staged);
            let result = self.step(op, &mut stack, &mut staged);
            let post = digest_state(&staged);
            self.trace.steps.push(TraceStep {
                step: self.trace.steps.len(),
                pc,
                opcode: op.clone(),
                pre_digest: pre,
                post_digest: post,
            });
            match result {
                Ok(v) => {
                    if matches!(op, OpCode::Halt) {
                        *state = staged;
                        self.trace.root = trace_root(&self.trace.steps);
                        return Ok(v);
                    }
                }
                Err(message) => return Err(VmError::Runtime { pc, message }),
            }
        }
        Err(VmError::Runtime {
            pc: code.ops.len(),
            message: "execution reached end without halt".into(),
        })
    }
    fn pop_num(stack: &mut Vec<Value>, pc: usize) -> Result<u64, String> {
        match stack.pop() {
            Some(Value::U64(n)) => Ok(n),
            _ => Err(format!("expected numeric stack value at {}", pc)),
        }
    }
    fn pop_loan(stack: &mut Vec<Value>, pc: usize) -> Result<String, String> {
        match stack.pop() {
            Some(Value::Loan(n)) => Ok(n),
            _ => Err(format!("expected loan at {}", pc)),
        }
    }
    fn step(
        &mut self,
        op: &OpCode,
        stack: &mut Vec<Value>,
        state: &mut FinancialState,
    ) -> Result<Value, String> {
        let pc = self.trace.steps.len();
        match op {
            OpCode::Push(n) => {
                stack.push(Value::U64(*n));
                Ok(Value::U64(*n))
            }
            OpCode::PushBool(b) => {
                stack.push(Value::Bool(*b));
                Ok(Value::Bool(*b))
            }
            OpCode::Binary(op) => {
                let right = stack
                    .pop()
                    .ok_or_else(|| "binary right operand missing".to_string())?;
                let left = stack
                    .pop()
                    .ok_or_else(|| "binary left operand missing".to_string())?;
                let result = match op {
                    crate::ast::BinOp::Add => Value::U64(match (left, right) {
                        (Value::U64(a), Value::U64(b)) => {
                            a.checked_add(b).ok_or("addition overflow")?
                        }
                        _ => return Err("arithmetic expects numbers".into()),
                    }),
                    crate::ast::BinOp::Sub => Value::U64(match (left, right) {
                        (Value::U64(a), Value::U64(b)) => {
                            a.checked_sub(b).ok_or("subtraction underflow")?
                        }
                        _ => return Err("arithmetic expects numbers".into()),
                    }),
                    crate::ast::BinOp::Mul => Value::U64(match (left, right) {
                        (Value::U64(a), Value::U64(b)) => {
                            a.checked_mul(b).ok_or("multiplication overflow")?
                        }
                        _ => return Err("arithmetic expects numbers".into()),
                    }),
                    crate::ast::BinOp::Div => Value::U64(match (left, right) {
                        (Value::U64(_), Value::U64(0)) => return Err("division by zero".into()),
                        (Value::U64(a), Value::U64(b)) => {
                            a.checked_div(b).ok_or("division error")?
                        }
                        _ => return Err("arithmetic expects numbers".into()),
                    }),
                    crate::ast::BinOp::Lt => Value::Bool(match (left, right) {
                        (Value::U64(a), Value::U64(b)) => a < b,
                        _ => return Err("comparison expects numbers".into()),
                    }),
                    crate::ast::BinOp::Gt => Value::Bool(match (left, right) {
                        (Value::U64(a), Value::U64(b)) => a > b,
                        _ => return Err("comparison expects numbers".into()),
                    }),
                    crate::ast::BinOp::Le => Value::Bool(match (left, right) {
                        (Value::U64(a), Value::U64(b)) => a <= b,
                        _ => return Err("comparison expects numbers".into()),
                    }),
                    crate::ast::BinOp::Ge => Value::Bool(match (left, right) {
                        (Value::U64(a), Value::U64(b)) => a >= b,
                        _ => return Err("comparison expects numbers".into()),
                    }),
                    crate::ast::BinOp::Eq => Value::Bool(left == right),
                };
                stack.push(result.clone());
                Ok(result)
            }
            OpCode::Select { then_ops, else_ops } => {
                let condition = match stack.pop() {
                    Some(Value::Bool(b)) => b,
                    _ => return Err("if condition must be boolean".into()),
                };
                let selected = if condition { then_ops } else { else_ops };
                let mut result = Value::Unit;
                for child in selected {
                    result = self.step(child, stack, state)?;
                }
                Ok(result)
            }
            OpCode::Trap(message) => Err(message.clone()),
            OpCode::DefinePolicy {
                name,
                parameter,
                predicate,
            } => {
                self.policies
                    .insert(name.clone(), (parameter.clone(), predicate.clone()));
                Ok(Value::Unit)
            }
            OpCode::CallPolicy(name) => {
                let value = Self::pop_num(stack, pc)?;
                let Some((parameter, predicate)) = self.policies.get(name).cloned() else {
                    return Err(format!("policy '{}' is not defined", name));
                };
                let mut child = Vm::new();
                child.vars.insert(parameter, Value::U64(value));
                let code = compile(&predicate);
                let result = child.execute(&code, state).map_err(|e| e.to_string())?;
                match result {
                    Value::Bool(true) => {
                        stack.push(Value::U64(value));
                        Ok(Value::U64(value))
                    }
                    Value::Bool(false) => {
                        Err(format!("policy '{}' rejected value {}", name, value))
                    }
                    _ => Err("policy predicate must return Bool".into()),
                }
            }
            OpCode::Function { params, body } => {
                let v = Value::Function {
                    params: params.clone(),
                    body: body.clone(),
                };
                stack.push(v.clone());
                Ok(v)
            }
            OpCode::Call(count) => {
                let mut args = Vec::with_capacity(*count);
                for _ in 0..*count {
                    args.push(
                        stack
                            .pop()
                            .ok_or_else(|| "call argument stack underflow".to_string())?,
                    );
                }
                args.reverse();
                let function = stack
                    .pop()
                    .ok_or_else(|| "call target missing".to_string())?;
                let Value::Function { params, body } = function else {
                    return Err("call target is not a function".into());
                };
                if params.len() != args.len() {
                    return Err("function argument count mismatch".into());
                }
                let mut child = Vm::new();
                for (name, value) in params.into_iter().zip(args) {
                    child.vars.insert(name, value);
                }
                let code = compile(&body);
                let result = child.execute(&code, state).map_err(|e| e.to_string())?;
                stack.push(result.clone());
                Ok(result)
            }
            OpCode::Load(n) => {
                let v = self
                    .vars
                    .get(n)
                    .cloned()
                    .ok_or_else(|| format!("unknown variable {}", n))?;
                stack.push(v.clone());
                Ok(v)
            }
            OpCode::Store(n) => {
                let v = stack
                    .pop()
                    .ok_or_else(|| "store stack underflow".to_string())?;
                self.vars.insert(n.clone(), v);
                Ok(Value::Unit)
            }
            OpCode::Pop => {
                stack.pop();
                Ok(Value::Unit)
            }
            OpCode::OracleRead(feed) => {
                let v = self
                    .oracle_values
                    .get_mut(feed)
                    .and_then(VecDeque::pop_front)
                    .ok_or_else(|| format!("oracle input unavailable for {:?}", feed))?;

                stack.push(Value::U64(v));
                Ok(Value::U64(v))
            }
            OpCode::Validate(policy) => {
                let v = Self::pop_num(stack, pc)?;
                if matches!(policy, PolicyId::PriceBounds) && v == 0 {
                    return Err(format!("validation failed: {}", v));
                }
                stack.push(Value::U64(v));
                Ok(Value::U64(v))
            }
            OpCode::ToAmount | OpCode::UnsafeAssumeTrusted => {
                let v = stack
                    .pop()
                    .ok_or_else(|| "conversion stack underflow".to_string())?;
                stack.push(v.clone());
                Ok(v)
            }
            OpCode::Mint(account) => {
                let n = Self::pop_num(stack, pc)?;
                abstract_interpreter::mint(state, account, n)?;
                Ok(Value::Unit)
            }
            OpCode::Transfer(from, to) => {
                let asset = match stack.pop() {
                    Some(Value::Asset(x)) => x,
                    _ => return Err("transfer expects asset".into()),
                };
                abstract_interpreter::transfer_asset(state, &asset, from, to)?;
                Ok(Value::Unit)
            }
            OpCode::CreateLoan {
                borrower,
                lender_pool,
                loan_id,
                collateral_asset,
                required_ratio,
            } => {
                let coll = Self::pop_num(stack, pc)?;
                let amount = Self::pop_num(stack, pc)?;
                abstract_interpreter::create_loan(
                    state,
                    borrower,
                    lender_pool,
                    loan_id,
                    amount,
                    collateral_asset,
                    coll,
                    *required_ratio,
                )?;
                stack.push(Value::Loan(loan_id.clone()));
                Ok(Value::Loan(loan_id.clone()))
            }
            OpCode::Repay {
                borrower,
                lender_pool,
            } => {
                let payment = Self::pop_num(stack, pc)?;
                let loan = Self::pop_loan(stack, pc)?;
                abstract_interpreter::repay(state, borrower, lender_pool, &loan, payment)?;
                stack.push(Value::Loan(loan.clone()));
                Ok(Value::Loan(loan))
            }
            OpCode::PriceUpdate => {
                let price = Self::pop_num(stack, pc)?;
                let loan = Self::pop_loan(stack, pc)?;
                abstract_interpreter::price_update(state, &loan, price)?;
                stack.push(Value::Loan(loan.clone()));
                Ok(Value::Loan(loan))
            }
            OpCode::Liquidate => {
                let loan = Self::pop_loan(stack, pc)?;
                abstract_interpreter::liquidate(state, &loan)?;
                Ok(Value::Unit)
            }
            OpCode::Halt => Ok(stack.pop().unwrap_or(Value::Unit)),
        }
    }
}

pub fn digest_state(s: &FinancialState) -> u64 {
    let mut h = 1469598103934665603u64;
    for (map, tag) in [
        (&s.balances, "b"),
        (&s.debts, "d"),
        (&s.receivables, "r"),
        (&s.collaterals, "c"),
    ] {
        let mut entries: Vec<_> = map.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(b.0));
        for (key, val) in entries {
            for b in key
                .as_bytes()
                .iter()
                .chain(tag.as_bytes())
                .chain(val.to_le_bytes().iter())
            {
                h ^= *b as u64;
                h = h.wrapping_mul(1099511628211);
            }
        }
    }
    h
}
pub fn trace_root(steps: &[TraceStep]) -> u64 {
    steps.iter().fold(0xcbf29ce484222325u64, |h, s| {
        h.rotate_left(5) ^ s.pre_digest.wrapping_mul(31) ^ s.post_digest
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{compiler::compile_source, state::FinancialState};
    #[test]
    fn compiles_and_executes_bytecode() {
        let c = compile_source("mint(alice, 10)").unwrap();
        let b = compile(&c.typed_ast);
        let mut vm = Vm::new();
        let mut s = FinancialState::new();
        assert_eq!(vm.execute(&b, &mut s), Ok(Value::Unit));
        assert_eq!(s.get_balance("alice"), 10);
        assert!(!vm.trace.steps.is_empty());
    }
    #[test]
    fn verifier_rejects_missing_halt() {
        assert!(verify(&Bytecode {
            version: 1,
            ops: vec![OpCode::Push(1)]
        })
        .is_err());
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionStatement {
    pub initial_state_digest: u64,
    pub program_digest: u64,
    pub final_state_digest: u64,
    pub trace_root: u64,
}

pub fn program_digest(code: &Bytecode) -> u64 {
    let mut h = 1469598103934665603u64;
    for op in &code.ops {
        let text = format!("{:?}", op);
        for b in text.as_bytes() {
            h ^= *b as u64;
            h = h.wrapping_mul(1099511628211);
        }
    }
    h
}

pub fn execution_statement(
    code: &Bytecode,
    initial: &FinancialState,
    final_state: &FinancialState,
    trace: &ExecutionTrace,
) -> ExecutionStatement {
    ExecutionStatement {
        initial_state_digest: digest_state(initial),
        program_digest: program_digest(code),
        final_state_digest: digest_state(final_state),
        trace_root: trace.root,
    }
}

pub fn verify_execution_statement(
    code: &Bytecode,
    initial: &FinancialState,
    final_state: &FinancialState,
    trace: &ExecutionTrace,
    statement: &ExecutionStatement,
) -> bool {
    statement.program_digest == program_digest(code)
        && statement.initial_state_digest == digest_state(initial)
        && statement.final_state_digest == digest_state(final_state)
        && statement.trace_root == trace_root(&trace.steps)
}

impl std::fmt::Display for VerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bytecode verification failed: {:?}", self)
    }
}
impl std::fmt::Display for VmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VM execution failed: {:?}", self)
    }
}
impl std::error::Error for VmError {}
