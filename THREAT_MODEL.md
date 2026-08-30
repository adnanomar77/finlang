# FinLang Threat Model

## Assets and trust boundaries

The protected assets are balances, ownership records, collateral, debt, receivables, loan status, bytecode, oracle inputs, and execution traces. Source text and oracle inputs are untrusted. The compiler boundary is crossed only after lexical, syntactic, and type checks. The VM boundary is crossed only after bytecode version and stack verification.

## Threats and controls

| Threat | Control | Verification |
|---|---|---|
| Loan copy or reuse | Linear type environment consumes loans | Negative compilation tests and VM value consumption |
| Double spending an asset | Ownership check before transfer | Integration test |
| Unauthorized transfer | Sender must own asset | Integration test |
| Plain oracle amount in sensitive operation | Verified type requires policy and source | Type checker |
| Trust escalation | Unsafe effect is rejected by protected policy | Policy test |
| Invalid bytecode | Version, halt, and stack verification | Verifier tests |
| Partial state mutation | Staged state commit | Rollback test |
| Arithmetic overflow/underflow | checked arithmetic and invariant checks | Arithmetic tests |
| Nondeterministic digest | Sorted map keys and explicit FIFO oracle queue | Determinism test |

## Residual risks

The implementation does not claim an independent security audit, a cryptographic hash suitable for adversarial commitments, a proof-assistant theorem, or a production ZK proof system. Those require external review and cryptographic design before handling real funds.
