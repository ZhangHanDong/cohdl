# M2 programmability fixtures

The `*.cohdl.txt` fixtures referenced by RFC-033's proposal history were
pre-M2 editing baselines; the constructs they sketched (consts, expression
array lengths, labelled loops) are now RUNNABLE language — no `.txt`
extension needed. The executed baselines live as regression tests:

- `tests/rfc033_scenarios.rs` — the RFC's three normative scenarios (LED
  chain N=1/2/10, RC channels with the capacitor-seven override and
  N=10→12 lock carry, nested `FilterBank<const N: Int>` with a
  board-level override through the boundary).
- `tests/rfc033_loops.rs` / `tests/rfc033_identity.rs` — frame hygiene and
  identity stability (insert-grow-rename).
- `tests/rfc033_arrays.rs` / `tests/rfc033_eval.rs` / `tests/rfc033_budget.rs`
  — the evaluator, computed lengths/selectors and the expansion budgets.

Note: the RFC's "executed baseline at `rc-workflow/` (main `0e3d770`)"
referred to a fixture tree that was never merged to this repository's
`docs/proposals`; the equivalent coverage is the RC scenario above and the
corpus audit recorded in `docs/compliance-report.md`'s RFC-033 ledger.
