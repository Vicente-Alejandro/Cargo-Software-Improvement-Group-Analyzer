# cargo-sig — Technical Roadmap

**Project:** Cargo Software Improvement Group Analyzer (`cargo-sig`)
**Vision:** Bridging the gap between high-performance systems and enterprise software quality.
**Core Principles:**
1. **Zero-Configuration & Ultra-Minimal:** `cargo sig` works out-of-the-box. We hand-roll the CLI parsing (`std::env::args`) and terminal styling (`owo-colors`) to maintain a microscopic dependency tree.
2. **Deep Modules:** Complex implementations (AST parsing) are hidden behind simple, unified interfaces.
3. **Data-Oriented Design & Concurrency:** Maximize performance using cache-coherent structures.
4. **Pure Rust:** No C dependencies. Leveraging native crates where strictly necessary.

---

## v0.1.0 — Foundation & Deep Modules
**Status:** Completed

- [x] **CLI Skeleton:** Implement the zero-configuration CLI manually. The default `cargo sig` command performs a standard check.
- [x] **Project Structure:** Establish the internal modular architecture (`analysis`, `churn`, `duplication`, `scoring`).
- [x] **Dependency Baseline:** Establish bare-metal baseline (removed `clap` and heavy macros, using `owo-colors`, `tree-sitter` and `tree-sitter-rust`).

## v0.2.0 — AST Volume Analysis
**Status:** Completed

- [x] **Volume Analysis (SIG Guideline 1):** Traverse the AST to measure Lines of Code (LOC) per function. Flag functions exceeding 15 lines.
- [x] **Interface Analysis (SIG Guideline 4):** Analyze function signatures to flag those with more than 4 parameters.
- [x] **Basic Reporting:** Output a clean, colored summary to the terminal.
- [x] **Data-Oriented Metrics:** Store extracted metrics in contiguous flat arrays for fast aggregation.

## v0.3.0 — Scoring & Concurrency
**Status:** Completed

- [x] **1-7 Star Scoring Engine:** Implement the math in `scoring/mod.rs` to map Volume/Complexity metrics to the 1-7 star scale.
- [x] **Cyclomatic Complexity (SIG Guideline 2):** Analyze AST branches (`if`, `match`, `while`, `for`) to flag units with complexity > 5.
- [x] **Parallel Processing:** Implement parallel AST parsing across multiple files using `rayon`.
- [x] **CI/CD Quality Gate:** Implement the logic for `--fail-below <1-7>` flag to fail the process.

## v0.4.2 — Hotspots (Coverage & Balance)
**Status:** Completed

- [x] **Git History Parser:** Use `std::process::Command` to read `git log` and calculate file churn.
- [x] **Coverage Ingestion:** Parse `.lcov` directly without `serde_json` to keep zero bloat.
- [x] **Component Balance (SIG Guideline 7):** Aggregate LOC by directory and check if any directory dominates >50%.

## v0.5.0 — Duplication Engine (SIG Guideline 3)
**Status:** Completed

- [x] **AST Subtree Hashing:** Traverse the `tree-sitter` AST to hash logic blocks and identify identical code clones.
- [x] **Duplication Rating:** Measure what percentage of the total codebase is duplicated.

## v0.5.1 — QoL & Audit Pass
**Status:** Completed

- [x] **Quality-of-life fixes and audit improvements** across the existing modules.
- [x] **crates.io readiness pass:** README and `Cargo.toml` metadata (repository URL, description) prepared for publication.

---

## v0.6.0 — Scoring Integrity Fix
**Status:** Completed

An audit of `scoring/mod.rs` confirmed that `categorize_risk()` only factors in `lines_of_code` and `cyclomatic_complexity` (SIG Guidelines 1 & 2), plus Duplication (Guideline 3, wired in during v0.5.0. Interface Size (Guideline 4) and Component Balance (Guideline 7) are computed and *reported* to the terminal, but never reach the star-rating formula. A function with 30 parameters can currently still score 7 stars.

- [x] **Fix:** wire Interface Size (Guideline 4, >4 parameters) into `categorize_risk()`.
- [x] **Fix:** wire Component Balance (Guideline 7, >50% LOC in one directory) into the scoring formula — currently console-only in `report/mod.rs`.
- [x] **Regression test:** extend the existing test suite (established in v0.4.0) with an assertion that every computed metric provably influences the final score, so this class of bug can't silently reappear when the next guideline is added.

This release is deliberately scoped to correctness only — no new features — so it can be reviewed and shipped as an atomic fix ahead of anything else in the v0.6 series.

## v0.6.1 — Module Coupling & Cohesion Engine (SIG Guidelines 5-6)
**Status:** Completed

- [x] Analyze `use` declarations (Fan-In / Fan-Out).
- [x] Detect circular dependencies natively.
- [x] Integrate Coupling limits into the 1-7 scoring formula.

## v0.6.2 — Structured Output (`--format json`)
**Status:** Completed

- [x] **JSON export:** serialize the full report — now covering all 6 implemented guidelines plus Churn × Coverage — for CI pipelines and third-party tooling.
- [x] **Open decision (resolve before implementing):** `serde_json` was removed during the zero-bloat rewrite. Decide explicitly between a small hand-rolled JSON writer (consistent with project philosophy, but needs correct string/number escaping) versus reintroducing a minimal JSON dependency. Don't default silently either way (we wont add a extern dependency, we will recreate it in RUST).

## v0.6.3 — Coverage Sub-score (SIG Guideline 9)
**Status:** Completed

- [x] **Standalone metric:** expose overall test coverage % as its own scored dimension. Today it only exists inside the Churn × Coverage cross-reference, not as a guideline in its own right.
- [x] **Scoring integration:** wire it into the star math alongside the other guidelines.

## v0.6.4 — Codebase Size Guideline (SIG Guideline 8)
**Status:** Planned

- [ ] **Total LOC threshold:** aggregate the per-function LOC already computed since v0.2.0 into a whole-codebase size check.
- [ ] **Scoring integration:** lowest priority of the batch — cheapest to add, smallest expected impact on the final score.

## v0.6.5 — Dogfooding & Release Readiness
**Status:** Planned

- [ ] **Self-validation:** add a CI step that runs `cargo sig --fail-below <N>` against `cargo-sig`'s own repository.
- [ ] **Full regression pass:** confirm all 6 implemented SIG guidelines plus Churn × Coverage are reflected in the final star math — closes the loop opened in v0.6.0.
- [ ] **Docs sync:** final pass across `README.md`, `ROADMAP.md`, and `Cargo.toml` to guarantee version, star scale, and dependency list all agree — the last gate before `cargo publish` to crates.io.

**Explicitly out of scope for v0.6:** SIG Guideline 10 (Write Clean Code) — too subjective to score without a dedicated lint engine of its own, and already largely covered by `clippy`.

---

*This document is actively maintained and reflects the current development priorities.*