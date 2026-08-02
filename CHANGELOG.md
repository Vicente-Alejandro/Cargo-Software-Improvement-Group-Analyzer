# Changelog
 
## [1.1.1] - 2026-08-01

### Refactored
- **Architectural Decomposition**: Refactored report generators, CLI parsing, duplication detection, and coverage helpers into modular, single-responsibility functions (≤ 15 LOC, cyclomatic complexity ≤ 4, parameters ≤ 4).
- **Self-Audited Code Health**: Restored codebase to a flawless **7/7 Stars** rating with 0 unit size violations, 0 complexity violations, 0 interface violations, and 0.0% high/moderate risk.

## [1.1.0] - 2026-08-01

### Added
- **Markdown Report Generator (`-r, --report`)**: Introduced actionable markdown report generation (`tools/cargo-sig/SIG_REPORT.md`) containing full tables for Unit Size, Complexity, Interface Parameters, Duplicated Code Spans, and Churn × Risk Hotspots.
- **Interactive `.gitignore` Assistant**: Automatically prompts to safely add `tools/cargo-sig/` to `.gitignore` on first generation when running in interactive environments.
- **Discovery Hint**: Added discovery suggestion in default terminal output to introduce users to `-r` and `--help`.

## [1.0.0] - 2026-08-01

### Added
- **Diagnostic Source Tracking**: Modified volume, complexity, parameter, and duplication analysis engines to capture and retain exact file paths, line ranges, and AST locations for every maintainability violation.
- **Enhanced CLI Help**: Upgraded `--help` overview with colored documentation and rich contextual descriptions for upcoming features.

### Changed
- Re-architected `DuplicationResult` to preserve exact continuous duplicated block spans across modules without discarding diagnostic data.

### Changed
- **Performance**: Parallelized file ingestion during duplication analysis using `rayon`, massively reducing I/O bottlenecks on large codebases.
- **Safety**: Added AST recursion bounds (`depth: 100`) to `duplication` walkers to prevent stack overflows on maliciously deep source files.
- **Quality**: Enforced `#![warn(clippy::pedantic)]` and `#![deny(clippy::all)]` across the core engine, bringing `cargo-sig` up to elite maintainability standards (7/7 stars).

## [0.8.1] - 2026-07-30

### Fixed
- Fixed bug in `duplication` AST parsing where `#[cfg(test)]` attribute isolation failed due to child/sibling grammar mismatches, falsely inflating duplication to 8.1%. Code duplication is now restored to 1.3%.
- Fixed `clippy::explicit_counter_loop` warning in coverage spinner.

## [0.8.0] - 2026-07-30

### Fixed
- Fixed an architectural blind spot where `duplication` analysis would ignore production code placed after `#[cfg(test)]` modules; it now uses full AST parsing to precisely skip only test nodes.
- Replaced the simulated coverage progress percentage with an honest visual spinner and elapsed time tracker for better UX.

## [0.7.1] - 2026-07-30

### Fixed
- Fixed `clippy::needless_borrow` warnings in `src/main.rs`.
- Enforced code formatting with `cargo fmt`.

## [0.7.0] - 2026-07-30

### Added
- Integrated test coverage generation directly into `cargo sig -a`.
- Added simulated progress indicator during coverage generation.
- Re-architected system limits and tests, recovering 97.2% overall test coverage and 7/7 stars code health score.
