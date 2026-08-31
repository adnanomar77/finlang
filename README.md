FinLang

FinLang is a deterministic, statically checked financial-contract language prototype designed for expressing financial state transitions with explicit resource, oracle, policy, and execution semantics.

The implementation provides a complete compilation and execution pipeline:

source .fin → lexer → parser → AST → type checker → typed AST → bytecode → verified VM → atomic state transition

The current semantics version is finlang-0.1.

Language

FinLang provides:

* statically checked financial expressions
* arithmetic and comparison operators
* conditional expressions
* lexical bindings with let ... in ...
* typed function definitions and calls
* financial state transitions
* explicit oracle inputs
* policy-based oracle validation
* verified oracle values
* explicit unsafe trust boundaries
* linear loans and financial assets
* deterministic execution semantics

Grammar

The expression grammar includes:

expr ::= let IDENT = expr in expr
       | if expr then expr else expr
       | fn(PARAMS) -> TYPE { expr }
       | expr ( expr-list )
       | mint(IDENT, expr)
       | transfer(IDENT, IDENT, expr)
       | oracleRead(feedA|feedB)
       | validate(expr, PriceBounds)
       | toAmount(expr)
       | unsafeAssumeTrusted(expr)
       | createLoan(IDENT, IDENT, IDENT, expr, IDENT, expr, NUMBER)
       | repay(IDENT, IDENT, expr, expr)
       | priceUpdate(expr, expr)
       | liquidate(expr)
       | NUMBER
       | IDENT

Comments begin with //.

Decimal literals such as 1.5 are supported for ratio-related operations.

Financial Operations

FinLang currently implements:

mint(...)
transfer(...)
createLoan(...)
repay(...)
priceUpdate(...)
liquidate(...)

The runtime enforces financial invariants including ownership, balances, collateral ratios, loan status, repayment limits, liquidation conditions, and checked arithmetic.

Oracle and Policy System

Oracle inputs are explicit execution inputs:

oracleRead(feedA)
oracleRead(feedB)

Oracle values can be validated through executable policies:

policy Minimum(x: Amount) {
    x >= 100
} in validate(oracleRead(feedA), Minimum)

The verified-oracle flow is:

Oracle
  ↓
validate(...)
  ↓
Verified
  ↓
toAmount(...)
  ↓
Amount

An explicit unsafe boundary is also available:

unsafeAssumeTrusted(oracleRead(feedA))

This operation bypasses normal verified-oracle conversion and is therefore represented as a distinct trust boundary.

Linear Resources

FinLang uses linear resource tracking for resources such as loans and financial assets.

A consumed linear resource cannot be consumed again.

The type system therefore rejects repeated use of resources such as an already-consumed loan and rejects programs that leave required linear resources unused.

Deterministic Virtual Machine

FinLang compiles typed programs into a deterministic bytecode representation executed by a verified VM.

The VM includes:

* bytecode version checking
* stack verification
* deterministic instruction execution
* execution tracing
* deterministic state digests
* program and bytecode commitments
* atomic state transitions
* rollback on failed execution

Given the same source program, bytecode, initial state, and oracle input sequence, execution is designed to produce the same result, final state, and trace root.

Execution Statements and Verification

The runtime produces execution evidence containing deterministic commitments over execution-related data.

The repository also contains cryptographic verification functionality for execution statements, including regression tests demonstrating that valid statements verify while modified statements or responses fail verification.

These mechanisms are part of the prototype and should not be interpreted as a production cryptographic system without independent security review.

Safety Properties

The current implementation includes:

* static type checking
* linear resource checking
* oracle/policy separation
* checked arithmetic
* financial invariant enforcement
* bytecode verification
* atomic commit/rollback semantics
* deterministic state hashing
* deterministic oracle queues
* execution trace commitments
* negative tests for invalid programs and invalid execution states

Verification and Testing

Run the complete Rust test suite:

cargo test --workspace

Check formatting:

cargo fmt --all -- --check

Run Clippy with warnings treated as errors:

cargo clippy --workspace --all-targets --all-features -- -D warnings

The repository also contains:

* property-based tests
* lexer robustness tests
* fuzzing infrastructure
* formal semantics documentation
* Coq proof artifacts
* security and threat-model documentation

CLI

After installation:

cargo install finlang

Check a program:

finlang check examples/loan.fin

Compile a program:

finlang compile examples/loan.fin

Run a program:

finlang run examples/loan.fin

Format a program:

finlang format examples/loan.fin

Run the program’s test workflow:

finlang test examples/loan.fin

The CLI also supports explicit oracle inputs:

finlang run program.fin --feedA 750

and named oracle inputs:

finlang run program.fin --oracle price 750

A live oracle demonstration is available through:

finlang run program.fin --live

Examples

The repository includes:

examples/arithmetic.fin
examples/loan.fin

The examples demonstrate deterministic arithmetic, conditional execution, financial state transitions, and verified oracle usage.

Project Structure

src/
├── ast.rs
├── lexer.rs
├── parser.rs
├── type_checker.rs
├── typed_ast.rs
├── compiler.rs
├── bytecode.rs
├── interpreter.rs
├── state.rs
├── effects.rs
├── abstract_interpreter.rs
└── zk.rs
tests/
├── full_program_tests.rs
├── functions.rs
├── language_features.rs
├── policies.rs
├── property_tests.rs
├── roadmap_tests.rs
└── vm_tests.rs
fuzz/
proofs/
docs/

Documentation

* Language Reference — Complete language specification.
* Getting Started — Installation and first-use guide.
* Formal Semantics — Execution model and safety obligations.
* Threat Model — Security assumptions and threat considerations.
* Security — Security reporting information.
* Security Audit — Internal security review scope.

Status

FinLang 0.1.3 is a research and engineering prototype.

The implementation is designed to provide a deterministic and statically checked foundation for financial-contract execution, including a typed language, linear resources, oracle policies, bytecode execution, atomic state transitions, execution evidence, and verification mechanisms.

It is not presented as production financial infrastructure or as having undergone an independent production security audit.