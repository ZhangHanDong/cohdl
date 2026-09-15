# M2 A syntax packet: loop labels and range endpoints

Date: 2026-09-12. **Deferred supporting material for RFC Appendix A; not a user-approved syntax decision or Accepted RFC.** A's scope is already selected. Per the user's subsequent direction, the active work is completing the RFC, while this packet is retained for later syntax discussion. It records the comparison work in readiness Task 1 without changing the Accepted language or claiming M2 execution.

**Recommendation: L1 + R1.** Require an authored label on every M2 loop, retain half-open `for a..b`, and keep existing inclusive/list fan-out. Do not add half-open fan-out in this amendment. Retain the draft's explicit minus and `%` lexical tradeoffs. This favors one loop form and the existing selector contract; it accepts redundant labels on placement-only loops and two endpoint conventions as disclosed costs.

Authority: the [English M2 working RFC](../rfc-draft-bounded-compile-time-programming.md), Accepted RFC-024/032 and the [A scope record](2026-09-12-m2-scope-a.md). Code facts below use main `0e3d7705ab393a5c64c20836540ad1bc2e49898e`.

## 1. What a label buys, and what it does not

```cohdl
// Proposed M2, not runnable on the baseline compiler.
for links: n in 0..(leds.len - 1) {
    net _: leds[n].DOUT, leds[n + 1].DIN
}
```

`links` identifies an authored operation site. Under the draft's L1 scheme, iteration 3 has frame `Board::__for_links_3`. Its anonymous net or called helper can own a generated path below that frame. The array element remains `Board::leds_3`; wiring or placing it never reparents it. A frame is not a subdesign, port interface, placement group or BOM component.

For wiring, an unchanged endpoint partition matters more than a particular printed net name. For effectful helper calls, generated physical paths also feed design.lock and designator allocation. Pure placement of already named objects does not create physical identity: a label there supplies a uniform syntactic site and diagnostics, not a necessary identity for the placed component.

The baseline uses a **global per-design** fn-call counter (`src/check/expand.rs:201`, `:1863`) and anonymous-net counter (`:1592`). M2 proposes iteration-owned counters; they are not already implemented. The draft's stable label/value frame does not remove the disclosed ordinal limit for calls inside one frame.

## 2. Three label policies

| Policy | Concrete rule | Benefit | Cost |
| --- | --- | --- | --- |
| **L1: all labelled — recommended** | Every circuit/layout loop uses `for LABEL: i in a..b`; missing label is a targeted syntax failure. Existing reserved-name/collision rules apply. | One grammar and one label/value frame rule, independent of body effects. Adding a helper or moving a loop between legal contexts does not require changing its header. | Even placement-only loops need a name, though target identity does not depend on that name. |
| L2: all labels optional | Both headers parse. An omitted label uses a lexical ordinal frame; explicit labels opt into edit-stable site identity. | Shortest common syntax; no label is needed just to obtain deterministic output from identical input. | Inserting/reordering unnamed sites can reassign generated objects; adding a label changes their paths. Tooling must explain which stability promise applies. |
| L3: labels optional only inside layout | Circuit loops require labels; loops lexically inside a layout block may omit them. The layout subset has no calls or object declarations. | Avoids naming many placement loops without transitive effect inference. | Two context-dependent header rules; moving a loop into a circuit body requires a label. Unnamed layout diagnostics still need site provenance. |

L2 is a viable deterministic design if its weaker edit-stability contract is accepted. An explicit alternative encoding would reserve disjoint tags: `__for_named_LABEL_VALUE` and `__for_auto_ORDINAL_VALUE`. Number only omitted-label loop sites in source order within the owning lexical body; layout blocks are transparent at that level, nested loop bodies start their own site sequence. Named siblings do not consume automatic ordinals. Each actual iteration still has its own helper/net counters. This encoding is comparison material, not the L1 spelling used by the RFC.

For L3, only unlabelled layout loops use automatic site ordinals; circuit loops retain explicit frames. Both frame kinds use L2's disjoint named/auto tags so an authored label cannot collide with an automatic site. A layout-loop body can contain const/place/nested loops only. Moving it into a circuit body requires an explicit label even if it still happens to perform only placement. No compiler inspection of callees is needed to decide whether a header is legal. Source span plus enclosing site/iteration distinguishes diagnostics; generated ordinal sites are not advertised as persistent object identity.

All three forms can use bounded parser lookahead: after `for IDENT`, a colon identifies the labelled header; otherwise an optional-label grammar expects contextual `in`. Fmt must preserve authored labels or their omission, never manufacture a persistent label as a cosmetic edit. The choice is a contract/teaching cost, not an inability to parse the shorter form deterministically.

Neither source-line numbers nor a hash of loop text solves the edit problem generally: inserting lines moves the former; editing a body changes the latter, while identical repeated bodies still require a tie-breaker. None is proposed as an invisible persistent-ID service.

### Edit consequences under the same workload

Assume two loops each call one effectful decoupling helper whose local capacitor is `c`. These paths are **design-model examples, not compiler output**.

| Edit | L1 labelled result | L2 omitted-label result |
| --- | --- | --- |
| Reorder `left` and `right` loops | `Board::__for_left_0::__fn0_decouple::c` stays associated with left | Original first site's `Board::__for_auto_0_0::__fn0_decouple::c` can now denote right; positional identity cannot preserve intent |
| Extend range `0..10` to `0..12` | Existing actual values 0–9 retain their frames | Existing values retain frames if the site's ordinal is unchanged |
| Rename label `left` to `supply` | Generated child paths deliberately change | Adding an explicit label replaces automatic paths; migration must inspect lock/topology/parts |
| Insert another helper before the existing helper inside one iteration | Original call ordinal shifts under the same frame | Same ordinal limit; optional labels do not repair it |
| Add a first helper to a circuit-body loop that currently contains only placement-subset layout | Named physical targets retain paths; the helper creates new child paths | Targets retain paths; new helper children get ordinal-site paths with the stated edit limits |
| Insert an earlier sibling (freshly labelled in L1, unnamed in L2) | Existing labelled frames unchanged; omitting the new label would instead be a syntax error | Later automatic site ordinals shift |

For L3, the L2 ordinal behavior concerns only placement-loop provenance: no physical helper children are legal there. It therefore avoids that physical-identity cost for omitted labels, at the price of the context-dependent syntax rule. This makes L3 the strongest alternative to L1, not an unsound option.

## 3. Three scenarios, unchanged circuit interfaces

| Scenario | Recommended L1 form | Why the label exists |
| --- | --- | --- |
| LED cascade | `for links: n in 0..(leds.len - 1)` | Distinguish each authored neighbor connection and its failing concrete index. A separate repeated decoupling helper additionally creates real parts. |
| Ten RC channels, then capacitor-seven override, then twelve | Named `channels` subdesign array; `for wiring: i in 0..channels.len`; layout `for placement: i in 0..channels.len` | Wiring has generated operation scope. Placement names an operation site while all `channels[i].c` identities remain ordinary subdesign paths. The explicit override stays outside the loop. |
| Nested FilterBank | Reuse its `wiring`/`placement` labels inside each actual bank activation | Enclosing subdesign paths separate bank instances; labels are unique in their lexical owner, not globally across a project. Shared N/R/C substitution and port rules are unchanged. |

These workflows do not need direct loop-body inst/subdesign. L1 does not turn fn into a pure function or make its locals externally placeable.

## 4. Range policies

**R1 — recommended:** half-open operation loops plus existing inclusive/list fan-out. A loop enters integers a through b−1; equal bounds mean no iterations, reversed bounds fail E1404. No loop step or inclusive for is added. An inclusive fan-out selects `a + k * step <= b`; default step is 1, and explicit step must be positive. Equal endpoints select one element; reversed endpoints and nonpositive steps fail E211. Actual selected indices must be in bounds (E202). Lists preserve authored order and duplicate membership until ordinary merging; no clipping occurs.

The distinction between creating one net and repeating statements does **not** force either endpoint spelling. R1 deliberately retains two conventions to avoid adding another selector form in this amendment. Its practical cost is `leds[0..=(leds.len - 1)].VDD` alongside `for ... in 0..leds.len`.

**R2 — alternative:** add half-open fan-out while retaining old inclusive/list forms. A concrete conservative contract would select `a + k * step < b`, allow positive `step`, and reject `a >= b` as E211. Thus half-open selectors would remain nonempty even though equal-bound loops are allowed to do nothing. Every actual selected index must be in bounds; just as for existing strided fan-out, a skipped endpoint is not an extra array lookup. This alternative gives `leds[0..leds.len].VDD` without introducing empty member sets or a zero-length hardware array. Allowing empty selections would be an additional decision, not an implied consequence of R2.

Both policies leave fan-out net-member-only for physical pins/subdesign ports. A single selector remains legal in ordinary reference positions; range/list selectors in place/nc/fn arguments remain E211. Neither policy invents a two-member minimum for a net: the baseline permits one physical member. Required pin/port obligations and electrical appropriateness are separate checks.

| Boundary example, array length 3 | R1 | R2 |
| --- | --- | --- |
| `[0..=2]` | Select 0,1,2 | Same legacy form |
| `[1..=1]` | Select 1 | Same legacy form |
| `[0..=3 step 2]` | Select 0,2; skipped endpoint 3 is not an access | Same legacy form |
| `[0..=3]` | E202 for selected 3 | Same legacy form |
| `[2..=1]` or `[0..=2 step 0]` | E211 | Same legacy form |
| `[0..3]` | Unsupported selector syntax; targeted syntax diagnostic | Select 0,1,2 |
| `[1..1]` | Unsupported selector syntax | E211, empty selector |
| `[0..4 step 5]` | Unsupported selector syntax | Select 0; bound checks apply to selected elements, not to skipped 4 |
| `for ... in 1..1` | No iterations | Same loop rule |

The last strided example exposes an authoring cost of preserving selection semantics: fan-out is an arithmetic selection of references, not an array-valued slice with a separately checked end offset. Requiring all written endpoints to lie inside the array would be a different rule and would reject some currently valid inclusive ranges. This packet does not silently adopt it.

R1's fmt keeps authored inclusive versus single/list form and whether `step` was explicit, then formats expressions by the shared expression rules. R2 would additionally preserve the authored half-open/inclusive delimiter; it must not normalize one into the other or rewrite a range into a singleton based on its evaluated value. Completion and diagnostics must describe the applicable endpoint. The current parser emits E010 for unsupported half-open syntax; parser recovery may add diagnostics, so the future targeted error must be tested at the original delimiter rather than pinned to an accidental cascade.

## 5. Lexical compatibility remains explicit

The [21-input lexer record](../../reviews/evidence/2026-09-11-m2-lexer-baseline.json) already measures the old tokenizer; this pass reads that evidence and does not claim to rerun the M2 parser.

- Keep the proposed standalone minus token outside strings/comments; parser context distinguishes subtraction from a prefix sign. Adjacent signed unit literals preserve their old text/span, while unary arithmetic uses canonical derived value text. Int MIN is admitted by combining a literal's direct sign with its magnitude before the Int range check.
- Keep `%` as proposed integer remainder when separated from a numeric token: `10 % 3`. Existing `10%` remains Tolerance; `10%3` stays E103. Fmt inserts binary-operator spacing but cannot remove this lexical authoring cost.
- Existing literal consumers still apply their unit/domain errors. None of these choices authorizes general unit arithmetic, coercion or type-directed retokenization.

## 6. Executed evidence and what remains

This pass ran the fixed baseline's 15 instance-array tests: all passed. It also ran ten synthetic existing-syntax probes through the real check pipeline, recorded in [range-baseline JSON](../../reviews/evidence/2026-09-12-m2-range-baseline.json), with source, observer source/build command and hashes. Inclusive/singleton/strided selections, duplicate membership and a one-physical-member net pass. Selected out-of-bounds, reversed/zero-step and unsupported half-open cases fail with the recorded diagnostics. The observer neither builds manufacturing artifacts nor implements a future loop.

The [declaration inventory](../../reviews/evidence/2026-09-12-m2-declaration-inventory.json) parsed 162 files across 60 packages without parser errors, finding **two nongeneric fn definitions and zero subdesign definitions**. Manifest/lock contents and hashes are recorded, but dependency resolution and new semantic validation were not run. This is input coverage for readiness Task 2, not its compatibility verdict. Even a future zero-delta library audit needs supplemental generic/subdesign cases; that corpus has too few reusable definitions to prove the correction by itself.

When this follow-up work resumes, review and record the L1/R1 and lexical recommendations, then synchronize their status in both RFCs. RFC Appendices A and B preserve the outstanding conditions; the [readiness plan](../../superpowers/plans/2026-09-12-m2-a-readiness.md) is a deferred checklist. The current work remains RFC authoring/review, and this packet does not activate compiler implementation.
