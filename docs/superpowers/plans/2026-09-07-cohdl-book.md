# CoHDL Book Implementation Plan

> **For agentic workers:** use the existing approved course roadmap and execute the documentation work task-by-task. This is the initial course edition; hardware lessons and future language features retain their actual verification status.

**Goal:** Organize the agreed Konnect-to-watch course as a locally readable mdBook and maintain CoHDL knowledge through real learning records.

**Architecture:** One book at `book/` with course, knowledge, learning-log and reference sections. mdBook consumes Markdown and real source includes; a standard-library Python checker validates generated local links. A separate CI workflow builds a downloadable reading copy.

**Tech Stack:** mdBook 0.5.3, Markdown, Python 3, existing CoHDL compiler.

## Global Constraints

- Scope and outline: `docs/book-outline.md`, derived from the confirmed watch roadmap.
- Compiler behavior and dependency contract remain unchanged.
- Current syntax and future M1–M4 proposals are labelled separately.
- No fabricated hands-on, manufacturing or physical-test completion.
- Generated HTML is ignored; existing site deployment is separate.

## Task 1: Book and course content

Files: `book/book.toml`, `book/src/SUMMARY.md`, `book/src/{course,cohdl,learning,appendix}/`, `book/README.md`, `book/theme/course.css`, `README.md`.

- [x] Add configuration, navigation and reading paths.
- [x] Write the first three practical lessons and subsequent task guides.
- [x] Add current-language chapters, reference links and learning-log workflow.
- [x] Include a self-contained teaching source from `book/examples/connections.cohdl`.
- [x] Review content against the source, confirmed scope and cross-chapter consistency; apply the three independent reviewers' corrections.

## Task 2: Build and maintain the book

Files: `book/tools/check_book.py`, `.github/workflows/book.yml`, `book/.gitignore`.

- [x] Add generated-link/fragment/resource and SUMMARY coverage checking.
- [x] Add an independent CI build and reading artifact, pinned to mdBook 0.5.3.
- [x] Run `mdbook build book` and `python3 book/tools/check_book.py`.
- [x] Run the example's check and fmt commands; expect exit 0.
- [x] Remove RETURN in a temporary copy, check for E701, then discard the temporary directory automatically.
- [x] Attempt Browser inspection: runtime reports no available browsers. Verify local HTTP responses and generated links/resources instead; interactive navigation/search remains unverified.
- [x] Keep learning state at the true current stage and report actual checks with the next lesson entry point.
