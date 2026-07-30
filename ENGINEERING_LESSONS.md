# Engineering Lessons & Architectural Insights

This document captures the core methodologies, architectural decisions, and software engineering philosophies discovered and applied during the creation of `cargo-sig` (v0.1.0 to v0.6.5). It serves as a foundational reference for future projects to ensure high-quality, maintainable, and pragmatic software development.

## 1. Software Quality Philosophies

### SIG (Software Improvement Group) Methodology
- **Objective Metrics Over Subjective Opinions:** Code maintainability is not an abstract concept; it can be measured using strict bounds.
  - **Volume (Guideline 1 & 8):** Codebase size matters. Large volume directly correlates with maintenance effort, but it should not *strictly* overshadow code health unless it reaches extreme monolithic sizes.
  - **Unit Size & Complexity (Guideline 2 & 4):** Functions must remain small (e.g., < 15 lines) and structurally simple (e.g., < 5 branches) to remain testable.
  - **Interfaces (Guideline 5):** Limit parameters (e.g., ≤ 4). High parameter counts indicate a violation of the Single Responsibility Principle.
  - **Coupling (Guideline 6):** Fan-out (outgoing dependencies) must be controlled, and circular dependencies must be strictly forbidden to allow modular independent testing.

### The CodeScene & Feathers Philosophy (Churn × Coverage)
- **Averages Hide Risk:** When evaluating a system, averaging the quality metrics of all files creates a false sense of security. A system with a perfect core but a toxic, highly-coupled, untested module is a high-risk system.
- **The "Bottleneck" Mathematics (AND Boolean):** Final system health must be gated by its weakest critical link. Using `Final Score = min(Code Health, Test Coverage)` correctly exposes risk instead of diluting it.
- **Risk = Complexity + High Churn + Low Coverage:** A complex file that never changes is a low priority. A complex file that changes every week without tests is a ticking time bomb. Weighting test coverage by **Churn** (commit frequency) prevents false alarms on dead code and focuses attention on active hotspots.

## 2. System Design & Rust Architecture

### Zero-Bloat & Dependency Minimalism
- **Rejecting Default Heavyweights:** While libraries like `serde` and `serde_json` are industry standards, adding them for a highly static, predictable JSON output adds unnecessary compile time and binary bloat.
- **Hand-rolled Solutions:** Writing a custom, sanitized JSON exporter natively via `std` string formatting requires careful escaping but yields a drastically smaller footprint.

### AST over Regex (`tree-sitter`)
- **Context-Aware Parsing:** Regular expressions are insufficient for structural code analysis. Utilizing `tree-sitter` allows for precise extraction of function bounds, parameter counts, and branching logic without being fooled by strings, comments, or macros.

### Parallel Processing (`rayon`)
- **Embrace Data Parallelism:** Disk I/O and parsing are expensive. Using `rayon` to convert standard iterators into parallel iterators (`into_par_iter()`) provides massive performance gains practically for free in Rust.

## 3. Workflow & Professionalism

### The Living Roadmap (`ROADMAP.md`)
- A project without a strict roadmap falls victim to scope creep.
- **Strict Adherence:** The roadmap must be the definitive source of truth. Features are not implemented unless they are planned, and versions are not bumped until all requirements for that tier are fulfilled.

### Atomic Commits & Semantic Versioning
- **Commit Granularity:** Commits must represent a single, logical unit of work (e.g., `feat:`, `fix:`, `docs:`, `ci:`). Avoid "kitchen sink" commits that bundle unrelated changes.
- **Transparency:** Never use `git commit --amend` to hide structural iterations. A clean, chronological commit history is invaluable for understanding the evolution of a codebase.

### Dogfooding (Self-Validation)
- Software must be subjected to its own standards. Integrating the tool into its own GitHub Actions CI pipeline (`cargo run --release -- sig --fail-below 5`) proves the tool's viability and enforces a strict quality gate on future development.
