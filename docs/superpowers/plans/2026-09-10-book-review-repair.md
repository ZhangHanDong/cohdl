# Book review repair plan

Scope approved by the user: apply the Book review recommendations, retain the demo-to-watch learning path, and make the verified RC workflow teachable. This is a documentation and evidence repair; no new language mechanism is implemented or accepted.

The chapter specification is `docs/book-outline.md`. Editing budget: one RC lesson of roughly 1,800–2,500 Chinese characters plus executable excerpts, one diagnostic guide of roughly 800–1,200 characters, and short navigation/status corrections. Explain observation, language representation, checks and remaining physical questions. Preserve actual session records and learner status.

- [x] Separate current-language chapters from language research in `book/src/SUMMARY.md`; mark course outlines and historical Chinese RFC content. Rewrite the current Chinese M2 overview against the English draft without choosing A/B.
- [x] Add `book/src/course/03-rc-workflow.md` and `book/src/cohdl/diagnostics.md`; link the course, route, index and cross-chapter trace. Explain actual current-syntax code and independently rerun the teaching experiments.
- [x] Record compiler source revision/version and executable hash in the RC verifier; run positive stages and isolated negative/intent probes. Preserve prior execution evidence in a separate JSON file.
- [x] Add an array-element internal-placement regression in `tests/subdesign.rs`; add the library compatibility inventory requirement to the English RFC. Do not claim that the proposed validator exists or that its corpus audit has run.
- [x] Record the actual work under `book/src/learning/`, update progress and book maintenance instructions, build/check the Book, then obtain the three read-only reviews required by the technical-writing skill and resolve their findings.

Validation: `cargo test --test subdesign`; `cargo fmt --check`; the RC verifier using the explicitly recorded main compiler; `mdbook build book`; `python3 book/tools/check_book.py`. The verifier checks topology and source-to-layout outputs, not analog response, board clearance, fabrication or learner mastery. No commit, publication or KiCad project mutation is part of this repair.
