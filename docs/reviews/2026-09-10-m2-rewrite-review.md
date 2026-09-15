# M2 capability-model rewrite review — 2026-09-10

> **后续依据校正（2026-09-10）：** 已拉取 main `0e3d770`，确认 `docs/design/rfc-032-subdesign.md` 为 Accepted 且收录于 note 10。此前“RFC-032 已关闭、不能作为 M2 依据”的判断混淆了同编号旧 harness 提案，现撤回。以下保留当时操作和验证事实，不再沿用其中的排除判断；最新决定见中英文 M2 草案的“main 基线与必须完成的组合修订”对应章节。

Scope: the English/Chinese draft at `docs/proposals/rfc-draft-bounded-compile-time-programming*.md`, rewritten after the user's tscircuit reference study and explicit question about the type-system connection. Three independent read-only reviews covered semantics, CoHDL coherence, and factual/translation consistency. This record is review evidence, not design acceptance.

The reviews identified these concrete issues; all were addressed in the draft:

| Issue | Resolution |
| --- | --- |
| A statement that every declaration creates one instance overlooked arrays; the acceptance row confused iteration count with array length | Specify one scalar per entered frame, L elements for a length-L array; two frames with one scalar plus a length-3 array yield eight real instances and eight instance work items |
| “Conditional topology deferred” contradicted zero/one-iteration local construction | Explicitly include count-controlled topology; defer general Bool/if, branch-selection/checking semantics and parts-selection policy |
| Empty-loop declaration checks could inherit known gaps in ordinary body validation | Explicitly require local duplicate/shadowing and currently decidable unit/trait/parameter-kind checks in new loop bodies, without recursively strengthening unchanged legacy helper interiors |
| A sentence incorrectly assigned detection of still-well-typed wrong topology to the compiler | Assign the check to independent application-specific topology comparison tests in both languages |
| Comparing independent-input scenario B with shared-input scenario C would assert false equivalence | State that each has its own reference; direct/helper/nested equivalence uses the same scenario C interface/topology |

The reviewers found the direct-inst scope, loop identity, explicit RFC-020 placement amendment, cohdl.dev/cohdl.ai boundary and exclusion of closed RFC-032 consistent in the reviewed draft. These assessments do not approve the language feature.

Current-syntax validation is recorded separately in `docs/proposals/fixtures/m2-programmability/rewrite-2026-09-10.json`: three reference checks passed, wrong-unit/missing-pin/wrong-kind probes failed, and deliberate output shorting passed check but changed net partitions. No proposed CoHDL source was executed. The implementation acceptance matrix remains future work.

## Follow-up: explicit type-system relationship

The user requested a dedicated explanation of how the proposed design relates to CoHDL typing. Added the bilingual section covering units, structural integers, trait-constrained device types, generic substitution, Pin/Instance reference kinds, value-dependent array checks and final connection obligations. Three scoped read-only reviews found no semantic or translation-contract conflict. Two Chinese wording suggestions were applied: distinguish generic resolution/binding from parsing, and describe invariant failures as errors decidable without binding unknown values. This is design review, not compiler execution.
