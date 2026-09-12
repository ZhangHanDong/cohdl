# M2 RFC comparison fixture

Current English revision (2026-09-12): Candidate A is selected; B is deferred. Earlier direct-local `.cohdl.txt` sources below are historical Candidate B comparison material, not A implementation targets. A retains ordinary effectful fn calls but rejects direct loop-body inst/subdesign declarations. The current-syntax real-part RC edit/growth workflow is recorded separately in [rc-workflow](rc-workflow/README.md); see the [scope decision](../../decisions/2026-09-12-m2-scope-a.md).

2026-09-10 correction: main `0e3d7705ab393a5c64c20836540ad1bc2e49898e` contains Accepted RFC-032 subdesign, now restored as an M2 basis. The earlier exclusion confused it with this branch's differently titled historical harness proposal. These fixtures cover physical arrays/fn, not the still-needed programmable subdesign workflow. Recorded binary/IR results retain their original baselines; the mutable main worktree has now advanced to `0e3d770`.

`reference.cohdl` is supported by main `ab44257`. It has ten synthetic
AddressableLED interfaces and one synthetic host, with explicit daisy-chain
links and positions. It is check-only: no procurement parts, no build or
hardware claim.

```sh
/private/tmp/cohdl-main-subdesign-ab44257/target/debug/cohdl \
  check docs/proposals/fixtures/m2-programmability/reference.cohdl --no-std --json
```

`proposed.cohdl.txt` is an earlier proposed M2 source, not an automatically current A fixture. The .txt suffix
keeps unimplemented syntax out of executable CoHDL fixture discovery. It has
NOT passed checking. `baseline-results.json` records the current reference
pass and proposed-source rejection, including source/selected-binary hashes.
Both files use design name `LedChain`, but must be compiled separately.
`reference-topology.json` records an observation of main's checked IR: all
net partitions, NC and placements, without part binding or allocation.
The future implementation must compare expanded connectivity, NC decisions,
physical instances and placements; different scoped net names are not by
themselves a topology difference.

This check-only fixture cannot verify design.lock or manufacturing bytes.
The RFC requires a separate real-part build fixture for those gates.

`review-2026-09-09.json` records two additional old-syntax probes used during
the principles review: repeated helper layout constraints and an uncalled
helper with duplicate local names. Both check successfully on the baseline;
they expose metering/checking boundaries that the proposal must define.
They do not execute proposed loops or a large amplification workload.

Baseline compiler checkout and book/document checkout are separate. Build
the selected main checkout before reproducing; do not assume the old
documentation branch compiler implements M2.

## Historical: 2026-09-10 capability-model rewrite

That revision proposed direct loop-local `inst`, parameterized fn
composition and a staged type/elaboration/assembly contract. These are future
features; the existing compiler is used only to check explicit references.

| Scenario | Current-syntax reference | Proposed source | Observed checked IR |
| --- | --- | --- | --- |
| LED chain | `reference.cohdl` | `proposed.cohdl.txt` | 11 instances, 12 nets, 42 connected pins, 1 NC |
| Independent-input RC channels | `channels-reference.cohdl` | `channels-proposed.cohdl.txt` | 13 instances, 7 nets, 19 connected pins, no NC |
| Shared-input nested RC bank | `nested-reference.cohdl` | `nested-proposed.cohdl.txt` | 11 instances, 5 nets, 17 connected pins, no NC |

The two RC scenarios have different interfaces; compare each with its own
future implementation. All devices are synthetic, unbound interfaces. These
checks do not verify manufacturing, designators or component suitability.
`rewrite-2026-09-10.json` records the actual compiler/source hashes, check
results and RC net partitions. It does not overwrite the earlier baseline.

Reproduce a reference check from the repository root:

```sh
cargo run -- check docs/proposals/fixtures/m2-programmability/channels-reference.cohdl --no-std --json
cargo run -- check docs/proposals/fixtures/m2-programmability/nested-reference.cohdl --no-std --json
```

The small read-only IR observer calls the real check pipeline without parts,
allocation or emitters. After building the root library, it can be run with:

```sh
cargo build
rustc --edition=2021 docs/proposals/fixtures/m2-programmability/observe-reference.rs --extern cohdl=target/debug/libcohdl.rlib -L dependency=target/debug/deps -o /private/tmp/cohdl-m2-topology
/private/tmp/cohdl-m2-topology docs/proposals/fixtures/m2-programmability/channels-reference.cohdl
```

The recorded mutation probes were made on temporary copies, using these exact
single replacements (none changes the proposed files):

| Reference | Replacement | Actual result |
| --- | --- | --- |
| nested | `receiver_cluster::<1kohm, 100nF>` → `receiver_cluster::<100nF, 100nF>` | fail, E112 plus baseline E405 cascades |
| channels | Delete `    net GND [gnd]: ground.GND, c1.B` and its newline | fail, E701 |
| nested | `(source.OUT, ground.GND)` → `(source, ground.GND)` | fail, E602 plus subsequent E701 |
| channels | `r1.B, c1.A, outputs[1].IN` → `r1.B, c1.A, outputs[1].IN, outputs[0].IN` | check passes; two filtered nets merge, leaving 6 instead of 7 partitions |

The final probe is deliberately still well-typed: it demonstrates why an
independent topology comparison is necessary. It is not a missing ordinary
compiler rule. Future A tests must additionally prove that these obligations
survive actual iteration, shared generic/array evaluation and ordinary effectful
fn/subdesign composition. Direct loop-body inst/subdesign belongs in A's
rejection tests; it is no longer a positive construction target.
