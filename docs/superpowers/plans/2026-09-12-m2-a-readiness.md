# M2 Candidate A: deferred acceptance and implementation preparation

Date: 2026-09-12. Status: **deferred supporting checklist — the active task is completing the RFC**. Per the user's subsequent direction, syntax comparison and declaration-validator development are follow-up material in RFC Appendices A and B. Task 1's comparison packet and Task 2's source/AST inventory are preserved; no syntax choice, prototype or semantic compatibility audit is claimed complete. This file is not the current execution plan. It replaces the [old construction plan](2026-09-10-m2-typed-construction.md).

**Goal when this work resumes:** make A's grammar choices and declaration-check compatibility cost concrete enough for formal acceptance, then derive an implementation plan from the accepted contract. Finish and review the RFC's capability/type/scenario contract first; this checklist does not require a prototype to complete that document work.

**Architecture:** add typed expressions to the existing unit-value, generic-substitution and expansion machinery. Ordinary physical and subdesign arrays supply named objects; operation loops retain effectful fn calls but reject direct inst/subdesign declarations. Placement inherits its design/subdesign/fn owner. Keep one final assembly and the four existing residual DRC rules.

**Tech stack:** the existing Rust crate and hand-written lexer/parser/checker; existing mdBook/Python documentation checks; existing standalone editor, Explorer and registry packages. No new compiler dependency.

**Authority:** [scope decision](../../proposals/decisions/2026-09-12-m2-scope-a.md), [English working RFC](../../proposals/rfc-draft-bounded-compile-time-programming.md), Accepted note 10 and RFC-007/024/032. The fixed comparison baseline is main `0e3d7705ab393a5c64c20836540ad1bc2e49898e`. A later implementation must record its actual base and recheck any intervening language changes.

## Task 1: settle spelling with observable consequences

Read: RFC Design §§2, 4, 5, 7, 10; `src/lex.rs`, `src/parse.rs`, `src/check/expand.rs`, `src/lock.rs`, `docs/reviews/2026-09-11-m2-minus-lexing.md`.

Created: [syntax comparison packet](../../proposals/decisions/m2-a-syntax-options.md), recommending all-labelled loops and existing inclusive/list fan-out. The recommendation is ready for review, not an accepted decision. [Actual baseline evidence](../../reviews/2026-09-12-m2-syntax-baseline.md) includes 15 existing tests and ten selector probes.

- [x] Compare mandatory labels with an optional-label rule on the same LED, RC and nested examples. Show generated-net/helper identities before and after loop reordering, range growth, label edits and adding a helper call. Existing ordinal fn-call identity limits must stay visible.
- [x] Specify how an optional rule identifies anonymous operation sites and reports errors; do not infer a public group or placement path from a label.
- [x] Compare retaining inclusive fan-out with adding half-open fan-out. For each, specify empty selection, step, bounds, member-count errors and canonical formatting. Keep loop semantics distinct from the one-net effect of fan-out.
- [x] Carry forward the measured lexer baseline: signed unit-literal text/span compatibility, Int MIN, no-space subtraction, unary arithmetic, tolerance suffix versus remainder. List deliberate source-compatibility costs rather than calling them formatting details.
- [ ] Record the chosen spelling and rationale through the language proposal process, then update English and Chinese together. Until then, labelled loops and inclusive fan-out remain the draft baseline.

Exit evidence: one explicit syntax decision with before/after examples and failure cases. Neither the A selection nor passing documentation checks closes this task.

## Task 2: prototype and audit uniform declaration validation

Read: `src/check/bodies.rs` (`check_fn_bodies`, `check_subdesign_body`, `check_one`, generic/reference helpers), `src/check/generics.rs`, `src/resolve.rs`, `src/check/mod.rs`, and `tests/subdesign.rs`, `tests/modules.rs`, `tests/deps.rs`.

Future isolated prototype: modify the existing declaration-check path; add focused regression cases for unused and called definitions to a new `tests/declaration_validation.rs`. Derive the prototype's detailed implementation steps before editing compiler code. This preparation document does not claim the prototype exists.

Create after execution: `docs/reviews/m2-a-declaration-audit.md` and `docs/reviews/evidence/m2-a-declaration-audit.json`.

**Inventory completed, audit not completed:** [fixed-baseline inventory](../../reviews/evidence/2026-09-12-m2-declaration-inventory.json) covers 60 packages and 162 source files, with two nongeneric fn definitions and zero subdesign definitions. It records manifest/lock/source hashes and actual parser results. Dependency verification, prototype verdicts and before/after semantic deltas remain unmeasured. The first checkbox below therefore remains open.

- [ ] Inventory every manifest/package under `lib/` and all supplied fn/subdesign definitions, including unused definitions. Record source hashes, exact dependency/lock identities, checker revision and prototype revision.
- [ ] Track a separate focused semantic corpus alongside the whole-library corpus: generic fn/subdesign binding, unused definitions, duplicates versus legal net continuation, trait/reference/port checks and cycle stages. Zero library subdesigns or generic helpers must not be counted as passing semantic coverage. Record each case's expected stage and source independently of package totals.
- [ ] Establish a typed abstract environment that distinguishes known kinds from unknown concrete values. Reject invariant duplicate/generic/unit/reference errors without fabricating a concrete generic value; preserve active fn-cycle and declaration-time subdesign-cycle rules.
- [ ] Probe an uncalled wrong-unit helper, an empty loop calling it, a correct generic forwarder, and an error dependent on an unknown actual index. The same declaration must receive the same static validation regardless of call reachability or caller M2 syntax.
- [ ] Run old and prototype validators against the same inventory and dependency set. Compare code, primary span, construct and message for each difference. Record package/definition totals, category totals even when zero, and every skipped/unresolved item. Skips prevent a complete-coverage result.
- [ ] For each newly rejected definition, provide the source location and a migration. Compare verdicts and artifact hashes for the buildable valid corpus; explain each intended new declaration error separately. Compile-only library packages must not be counted as emitted board evidence.
- [ ] Review the measured compatibility cost before approving this amendment. Today's passing tests alone cannot substitute for the audit.

The evidence JSON must identify both compiler revisions, the source/dependency inventory, per-package coverage and old/new diagnostics, categorized differences, exclusions and available artifact comparisons. It contains observations only after the commands run; no placeholder zero may be presented as a measured result.

Exit evidence: reproducible before/after audit with full definition coverage, accounted-for deltas and a governance decision on compatibility. The audit may start independently of Task 1 where it uses existing syntax; future-loop probes require the agreed syntax and prototype support.

## Task 3: prepare formal acceptance

Read/update through acceptance: `docs/design/10-language-specification.md`, the affected RFC-007/024/032 documents, `docs/error-codes.md`, `docs/apidocs.md`, `docs/compliance-report.md`, and the eventual assigned RFC/decision record. Do not invent an available RFC number or treat E140x as already allocated.

- [ ] Attach Tasks 1–2 evidence and the three scenario oracles to the final proposal.
- [ ] Reconcile the existing fn cross-package wording and RFC-032 hierarchy/tooling obligations explicitly; do not treat a flat public IR as fulfilling a required hierarchy model.
- [ ] Record the accepted array/loop contexts, E1307 precedence, all-graph budget trigger, empty-body checking boundary and docs schema-version transition as one coherent contract.
- [ ] Obtain formal acceptance under repository governance and synchronize normative text in that change. Scope selection alone leaves all these documents unchanged.

Exit evidence: Accepted RFC and corresponding normative updates. If a decision changes the scope, revise this preparation plan and both translations before implementation planning continues.

## Task 4: derive the implementation plan after acceptance

The following is a dependency map, **not an activated coding checklist**. Split it into failing contract tests, implementation steps and actual verification commands once Tasks 1–3 have closed.

| Order | Existing seams | Required result and meaningful tests |
| --- | --- | --- |
| 1 | `src/lex.rs`, `src/ast.rs`, `src/parse.rs`, `src/check/generics.rs` | One typed Int/Length expression model and shared fn/subdesign Int binding. Add expression tests for MIN/MAX, precision, original literal text, lexical adjacency and parameter-kind failures. |
| 2 | `src/check/bodies.rs`, `src/check/expand.rs`, `src/ir.rs`, `src/lock.rs` | Const/array dependency graph; A owner/context rejection including empty loops; inherited placement and per-iteration provenance; full fn/subdesign graph budget activation. Add loop tests; extend `tests/inst_array.rs`, `tests/subdesign.rs`, `tests/layout.rs`. |
| 3 | `src/diag.rs`, `src/fmt.rs`, `src/lsp.rs`, `editors/vscode/syntaxes/cohdl.tmLanguage.json`, `explorer/extractor/src/` | Preserve diagnosable expansion context and semantic formatting. Extend `tests/json_output.rs`, `tests/error_registry.rs`, `tests/fmt.rs`, `tests/lsp.rs`, editor grammar tests and Explorer model tests. No second evaluator in tooling. |
| 4 | `src/emit/docsjson.rs`, `registry/src/worker/apidocs.ts`, `registry/src/ui/apidocs-model.ts`, `registry/src/ui/apidocs.tsx` | Versioned symbolic API bodies and const Int descriptors; version-1 behavior preserved. Extend `tests/apidocs.rs`, `registry/test/apidocs-model.test.ts`, `registry/test/apidocs.test.ts`; prove unsupported versions fail explicitly. |
| 5 | `docs/proposals/fixtures/m2-programmability/`, `tests/`, `book/` | Implemented A fixtures for LED, real-part RC ten→override seventh→twelve, and nested reuse. Compare complete endpoint partitions/NC/parts/placements against independent explicit references. Test budgets and deliberate wrong-but-well-typed shorting; preserve legacy designators and artifact bytes. |

All five rows are one release scope, including subdesign and the symbolic docs contract. Direct loop-body inst/subdesign remains a rejection test. Pure-fn migration, M3 component selection and M4 additional electrical contracts are separate proposals.

## Verification for the preparation documents

From the repository root, after editing:

```sh
mdbook build book
python3 book/tools/check_book.py
git diff --check
```

Also compare English/Chinese heading structure, code blocks, inline technical tokens, diagnostic codes, table shape and links; review translated prose for meaning. These checks validate documents, not the M2 compiler contract. The current RC executable sources were not changed by the A scope selection.
