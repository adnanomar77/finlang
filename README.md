# FinLang

[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.22181786.svg)](https://doi.org/10.5281/zenodo.22181786)

FinLang is a small statically checked financial-contract language prototype. The implementation now provides a complete text pipeline:
> source `.fin` → lexer → parser → AST → type checker → typed AST → atomic runtime

## Grammar

The expression grammar is:

```text
expr ::= let IDENT = expr in expr
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
       | NUMBER | IDENT
```

Comments begin with `//`. Decimal ratios such as `1.5` are accepted. The current semantics version is `finlang-0.1`.

## CLI

```bash
cargo run --bin finlang -- check examples/loan.fin
cargo run --bin finlang -- compile examples/loan.fin
cargo run --bin finlang -- run examples/loan.fin
cargo run --bin finlang -- format examples/loan.fin
cargo run --bin finlang -- test examples/loan.fin
```

`check` runs lexical, syntactic, and static checks. `compile` prints a stable debug representation containing the semantics version. `run` executes against a deterministic demonstration state and commits state changes only when the complete execution succeeds. Runtime errors leave the original state unchanged.

## Safety properties implemented

The type checker distinguishes plain amounts from oracle-derived verified amounts, tracks linear loans and assets, consumes loans on use, rejects unused linear resources, and rejects unsafe or untrusted effects for protected financial operations. The runtime validates ownership, balances, collateral ratios, loan status, repayment limits, and liquidation conditions. Balance updates use checked arithmetic, and `FinancialState::checked_net_value` rejects overflow and negative net value.

## Verification

Run the complete verification suite with:

```bash
cargo fmt -- --check
cargo test --all-targets
cargo clippy --all-targets --all-features -- -D warnings
```

Formal proofs, fuzzing engines, a bytecode VM, cryptographic commitments, persistent storage, and production security audits are not implied by this prototype and require additional dedicated implementation and review.

## Documentation

- [Language Reference](docs/language.md) — Complete FinLang language reference.
- [Getting Started](docs/getting-started.md) — Installation and first-use guide.
- [Formal Semantics](FORMAL_SEMANTICS.md) — Execution model and safety obligations.
- [Threat Model](THREAT_MODEL.md) — Security assumptions and threat considerations.
- [Security](SECURITY.md) — Security reporting information.
