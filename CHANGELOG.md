# Changelog

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
