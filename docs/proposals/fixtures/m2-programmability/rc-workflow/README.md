# Existing subdesign RC layout workflow

These are literal **current-syntax** references for the English M2 review, using main `0e3d7705ab393a5c64c20836540ad1bc2e49898e` (CoHDL 0.7.0). They contain no proposed expressions or loops. The three sources describe ten channels, ten with capacitor seven overridden, and twelve retaining the override. `verify.py` builds in a temporary project and carries the same `design.lock` between those edits; it never rewrites the lesson project.

Each channel has one real 1kohm resistor, one real 100nF capacitor and one six-pin connector, using exact locked library parts. P1/P2 provide input/output, P5 joins shared ground and P3/P4/P6 are explicit nc. This fixture replaces the RFC's synthetic input/output/ground interfaces with connectors: its total component count is therefore 3N, not the synthetic example's 4N+1. The passive-channel topology and independent layout task are the correspondence being checked; interface electrical roles are not identical models.

Run from the repository root with Python 3.11+ and a compiler built from the stated main:

```sh
cargo build --manifest-path /private/tmp/cohdl-main-subdesign-ab44257/Cargo.toml --locked --offline
git -C /private/tmp/cohdl-main-subdesign-ab44257 rev-parse HEAD
python3 docs/proposals/fixtures/m2-programmability/rc-workflow/verify.py \
  --compiler /private/tmp/cohdl-main-subdesign-ab44257/target/debug/cohdl \
  --compiler-source /private/tmp/cohdl-main-subdesign-ab44257 \
  --record /tmp/cohdl-rc-workflow-results.json
```

The recorded execution verifies successful builds, complete physical net partitions, BOM manufacturer/MPN/footprint and per-group designator membership, placements, unchanged surviving designators and connectivity restricted to surviving endpoints. Only capacitor seven changes during the override edit; nothing already present moves during growth. Shared ground legitimately gains new endpoints. Each stage is built twice with identical input state and compared by artifact/lock SHA-256. The results record compiler/input/source hashes and per-stage counts and placements.

Expected ten/twelve results: 30/36 physical components, 20/24 passives, 21/25 nets and 70/84 connected pad endpoints. The seventh capacitor moves from (61mm, 15mm) to (62mm, 17mm). The sources also pass current `cohdl fmt --check` after canonical formatting.

This checks the existing composition/layout/lock baseline. It does **not** execute Candidate A or B, validate M2's budget/validator, prove analog response or establish placement clearance, routing or manufacturing readiness. It supplies no evidence that direct loop-local construction is necessary.


The Book repair rerun is preserved in `book-results-2026-09-10.json`; the earlier `results-2026-09-10.json` is unchanged. The new record explicitly includes checkout commit, tracked changes, compiler version, executable hash and verifier hash. `--record` requires `--compiler-source`. That argument is the caller's build-source declaration, not a proof linking arbitrary binaries to a checkout: build the specified checkout first. The historical directory suffix `ab44257` is not its current Git HEAD; this run verified clean main `0e3d7705ab393a5c64c20836540ad1bc2e49898e`.

Every check/build must also preserve the original dependency-lock hash, including the first build.

The verifier now also checks five isolated teaching probes from the original ten-channel source: a Voltage in a placement Length field (E1007), missing capacitor ground (E701), missing external output (E1302/E701), out-of-bounds placement (E202), and an unintended output short (check succeeds, 21 partitions become 20). The last probe verifies the exact merged endpoint sets, not just a count. These are expected outcomes; all must hold for the verifier to exit zero. The Book lesson is `book/src/course/03-rc-workflow.md`.
