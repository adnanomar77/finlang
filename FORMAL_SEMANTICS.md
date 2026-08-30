# FinLang Formal Semantics (finlang-0.1)

## State transition

A successful execution is modeled as `⟨S, e⟩ → ⟨S', v⟩`. The public atomic runtime and VM implement transaction semantics: the transition is evaluated against a clone of `S`; `S'` is committed only if every step succeeds.

## Static safety

`oracleRead(f)` has type `Oracle(USD,f)`. `validate(o, PriceBounds)` consumes the untrusted oracle effect and produces `Verified(USD,PriceBounds,f)`. `toAmount` converts only a matching `Verified` value. Loans and linear assets are removed from the typing environment when referenced, which makes a second use ill-typed.

## Financial transitions

`mint(a,n)` adds `n` to the balance of `a` using checked addition. `transfer(a,b,x)` changes ownership only when `x` exists and is owned by `a`. `createLoan` debits the lender pool, credits the borrower, locks collateral, records debt and receivable, and requires the configured collateral ratio. `repay` transfers payment from borrower to lender pool and closes the loan exactly at zero debt. `priceUpdate` recomputes loan status. `liquidate` is legal only for a liquidatable loan and closes the debt while distributing covered debt and surplus.

## Progress and preservation obligations

The implementation includes executable checks for operation preconditions, stack verification, checked arithmetic, type checking, and state invariants. These are machine-checked runtime assertions and regression tests. A mathematical proof in a proof assistant is not claimed by this document; such a proof would be a separate artifact.

## Determinism

Oracle values are explicit FIFO inputs. HashMap state digests sort keys before hashing. Given the same source, bytecode, initial state, and oracle queue, the VM must produce the same final state and trace root; this property is covered by regression tests.
