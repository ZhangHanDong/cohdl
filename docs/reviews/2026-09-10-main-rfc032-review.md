# Main RFC-032 authority correction review — 2026-09-10

Scope: restore Accepted `docs/design/rfc-032-subdesign.md` as the M2 baseline after the user's explicit instruction to pull main and use its current RFC-032. The historical `docs/proposals/rfc-032-proof-gated-agent-harness.md` is a different proposal with a colliding number. Its closed-PR status does not revoke subdesign.

Executed `git pull --ff-only origin main` in the clean detached worktree `/private/tmp/cohdl-main-subdesign-ab44257`: advanced from `ab44257f8fc017ee97eb7228b601007f929cbaa3` to `0e3d7705ab393a5c64c20836540ad1bc2e49898e`. RFC-032, note 10, compiler source and subdesign tests have no changes across these two commits. The main worktree remains clean. The documentation branch was already at `89e891d8837e3f359460de99e442115b18907acc` at turn start; this pass did not merge into it.

Three independent read-only reviews covered semantic/source accuracy, CoHDL coherence and plan status, and bilingual/Book factual consistency. All reported no material findings. They specifically verified the separation of Accepted subdesign from Proposed M2, the preserved effectful fn contract, and the explicit unresolved composition/placement/budget requirements. This review does not accept M2 or settle pure-fn migration.

Executed verification:

- Latest main: `cargo test --test subdesign --offline --locked` — 24 passed, 0 failed, 0 ignored. No full-suite or hardware claim.
- English/Chinese RFC: source SHA-256, code blocks, heading levels, table-row counts and diagnostic-code sets match.
- `mdbook build book` — passed. One earlier attempt raced the running Book server while copying an asset; the repeat completed successfully.
- `python3 book/tools/check_book.py` — passed: 47 chapters, 50 HTML pages, 2485 local links/assets/fragments.
- Live Chinese RFC at `http://127.0.0.1:3001/cohdl/programming-rfc-zh.html` serves the new main SHA and composition-revision status.
- `git diff --check` — passed; compiler, tests, Cargo files and Accepted design files unchanged in the documentation branch.

The existing M2 implementation plan now explicitly requires revision before execution. Historical experiment results retain their original compiler/source baselines; proposed CoHDL fixtures were not executed.
