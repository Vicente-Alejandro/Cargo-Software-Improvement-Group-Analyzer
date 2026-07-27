# cargo-sig — Technical Roadmap

**Project:** Cargo Software Improvement Group Analyzer (`cargo-sig`)
**Vision:** Bridging the gap between high-performance systems and enterprise software quality.
**Core Principles:**
1. **Zero-Configuration:** `cargo sig` should work out-of-the-box on any Rust project. Flags are strictly for advanced overrides.
2. **Deep Modules:** Complex implementations (AST parsing, Git history) are hidden behind simple, unified interfaces.
3. **Data-Oriented Design & Concurrency:** Maximize performance using cache-coherent structures and parallel processing (`rayon`).
4. **Pure Rust:** No C dependencies. Leveraging native crates like `gix` and `jscpd-rs` for frictionless installation.

---

## v0.1.0 — Foundation & Deep Modules
**Status:** In Progress

- [ ] **CLI Skeleton:** Implement the zero-configuration CLI using `clap`. The default `cargo sig` command should perform a standard check.
- [ ] **Project Structure:** Establish the internal modular architecture (`analysis`, `churn`, `duplication`, `scoring`).
- [ ] **Dependency Baseline:** Upgrade to `tree-sitter` (latest), `clap` v4.6+, replace `git2` with `gix` (gitoxide), and add `rayon` & `jscpd-rs`.
- [ ] **Volume Analysis (SIG Rule 1):** Traverse the AST to measure Lines of Code (LOC) per function. Flag functions exceeding 15 lines.
- [ ] **Interface Analysis (SIG Rule 4):** Analyze function signatures to flag those with more than 4 parameters.
- [ ] **Basic Reporting:** Output a clean, colored summary to the terminal.

## v0.2.0 — Concurrency, Complexity & Automation
**Status:** Planned

- [ ] **Parallel Processing:** Implement parallel AST parsing across multiple files using `rayon`.
- [ ] **Data-Oriented Metrics:** Store extracted metrics in contiguous flat arrays for fast aggregation.
- [ ] **Cyclomatic Complexity (SIG Rule 2):** Analyze AST branches (`if`, `match`, `while`, `for`) to flag units with complexity > 5.
- [ ] **Component Balance:** Analyze module-level coupling and boundaries (e.g., dependencies between internal crates/modules).
- [ ] **CI/CD Quality Gate:** Implement `--fail-below` flag and track the project's own DORA metrics using GitHub Actions.

---

*This document is actively maintained and reflects the current development priorities.*
