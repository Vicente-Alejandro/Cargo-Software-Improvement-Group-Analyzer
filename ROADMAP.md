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

- [x] **Volume Analysis (SIG Rule 1):** Traverse the AST to measure Lines of Code (LOC) per function. Flag functions exceeding 15 lines.
- [x] **Interface Analysis (SIG Rule 4):** Analyze function signatures to flag those with more than 4 parameters.
- [x] **Basic Reporting:** Output a clean, colored summary to the terminal.
- [x] **Data-Oriented Metrics:** Store extracted metrics in contiguous flat arrays for fast aggregation.

## v0.3.0 — Scoring & Concurrency
**Status:** Completed

- [x] **1-7 Star Scoring Engine:** Implement the math in `scoring/mod.rs` to map Volume/Complexity metrics to the 1-7 star scale.
- [x] **Cyclomatic Complexity (SIG Rule 2):** Analyze AST branches (`if`, `match`, `while`, `for`) to flag units with complexity > 5.
- [x] **Parallel Processing:** Implement parallel AST parsing across multiple files using `rayon`.
- [x] **CI/CD Quality Gate:** Implement the logic for `--fail-below <1-7>` flag to fail the process.

## v0.4.2 — Hotspots (Coverage & Balance)
**Status:** Completed

- [x] **Git History Parser:** Use `std::process::Command` to read `git log` and calculate file churn.
- [x] **Coverage Ingestion:** Parse `.lcov` directly without `serde_json` to keep zero bloat.
- [x] **Component Balance:** Aggregate LOC by directory and check if any directory dominates >50%.

## v0.5.0 — Duplication Engine (SIG Rule 3)
**Status:** Planned

- [ ] **AST Subtree Hashing:** Traverse the `tree-sitter` AST to hash logic blocks and identify identical code clones.
- [ ] **Duplication Rating:** Measure what percentage of the total codebase is duplicated.

---

*This document is actively maintained and reflects the current development priorities.*
