# M2 syntax comparison: measured baseline and audit coverage

Date: 2026-09-12. Source: clean tracked main `0e3d7705ab393a5c64c20836540ad1bc2e49898e` in the existing temporary baseline checkout. These observations support the [syntax packet](../proposals/decisions/m2-a-syntax-options.md); they do not accept or implement its recommendations.

## Executed observations

`cargo test --manifest-path /private/tmp/cohdl-main-subdesign-ab44257/Cargo.toml --locked --offline --test inst_array` passed all 15 tests. `cargo build --lib --locked --offline` on the same manifest refreshed the public library used by a temporary Rust observer. The observer calls the actual lexer/parser for inventory and `check_files_in_with_deps` with no dependencies for synthetic selector probes. Its exact source, rustc invocation/version and executable/rlib hashes are embedded in both JSON reports.

| Evidence | Measured result | Limit |
| --- | --- | --- |
| [Range probes](evidence/2026-09-12-m2-range-baseline.json) | Ten actual check inputs: five valid, five invalid, all matched their stated expected outcome | Synthetic unbound Contact device; no build, layout or M2 evaluation |
| [Declaration inventory](evidence/2026-09-12-m2-declaration-inventory.json) | 60 manifests, 162 source files, 12,791,522 source bytes; zero parser errors/unowned files | Parsing only; manifest/lock identities recorded but dependency resolution/hash validation not executed |
| Reusable declarations in that inventory | Two public nongeneric fn definitions, zero subdesign definitions | No call-reachability inference; zero definitions of a kind is absent coverage, not success for its semantic rules |

The functions are `decoupling_100n` and `bulk_10u` in `lib/passive/src/circuits.cohdl`. The observer uses AST item kinds rather than text matching; comments and use sites do not inflate the definition count. It scans every supplied source, regardless of whether a board calls those functions. Recorded declaration offsets locate the leading declaration token (`pub` for these functions), not an asserted full-body span.

## Details that affect the proposal

- `ps[0..=3 step 2].A` on a three-element array succeeds: only 0 and 2 are selected. The baseline's `IndexSel::indices` / `array_bounds` checks selected elements; it does not separately reject the unused upper endpoint. A compatibility promise must account for this.
- A one-physical-member passive net succeeds. Do not invent a two-member rule when comparing optional empty fan-out policies. None of these observations prove useful electrical connectivity or hardware suitability.
- Reversed/zero-step selectors emit E211 with parser-recovery E010; unsupported `[0..3]` emits E010 and a recovered E202. These are full observed diagnostics, not a recommended future cascade contract.
- `fn` and anonymous-net ordinals are currently design-global in the expander; per-iteration counters are a proposed M2 change. The syntax packet keeps those facts separate.

## Consequence for the declaration audit

The whole-library audit remains necessary to measure compatibility against shipped material, but cannot alone prove generic/subdesign checking: this snapshot contains no subdesign definitions and neither helper has generics. Retain the complete library inventory and additionally track a separate focused corpus covering uncalled/called fn, subdesign bodies, abstract unit/trait forwarding, duplicate locals, legal repeated nets, ports, active fn cycles and recursive subdesign containment. Use existing `tests/subdesign.rs` and generic/declaration test cases as sources; give newly added probes their own expected stage and source references.

The report for the future prototype must distinguish package input coverage, semantic-case coverage and actual differences. No semantic delta or successful prototype audit is reported here. Empty-loop/value-binding tests require prototype support for the agreed M2 grammar; existing-syntax declaration checks can proceed earlier.

## Independent document review and verification

Three read-only reviewers checked semantics, source/evidence facts, and bilingual/Book consistency. Three distinct corrections were applied: L1 must reject an unnamed inserted sibling; adding a helper requires a circuit-body loop rather than a layout-body loop; recorded declaration spans begin at `pub` in the two inventory functions. No other blocking issue was reported. The recommendation remains open for syntax decision.

The English/Chinese RFCs match at 660 lines, 43 headings and six identical code blocks, with matching technical tokens, error codes, URLs and table structure. All recorded source/manifest/lock/observer/library hashes were rechecked. Book build/link verification passed with 63 chapters, 66 HTML pages and 3245 local links/assets/fragments; an isolated output directory was used to avoid the live preview rebuilding the directory during the check. `git diff --check` passed. No compiler or executable teaching source changed.
