<div align="center">

# Cargo Software Improvement Group Analyzer

**Local maintainability rating for Rust projects — SIG methodology, zero SaaS.**

[![Crates.io](https://img.shields.io/crates/v/cargo-sig.svg?style=flat-square&color=orange)](https://crates.io/crates/cargo-sig)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](./LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)

`tree-sitter` AST analysis · SIG 10-guideline checks · Churn × Coverage hotspots — in one command, with a 1–7 star rating.

**Current version:** `0.8.2`

</div>

---

## Overview

`cargo-sig` is a zero-configuration Cargo subcommand that rates the maintainability of a Rust codebase using the [Software Improvement Group (SIG)](https://www.softwareimprovementgroup.com/) model — the same system behind Better Code Hub — cross-referenced with a Churn × Coverage hotspot analysis inspired by Michael Feathers' *Working Effectively with Legacy Code*.

It solves a specific problem: giving any Rust project a **single, comparable maintainability score**, computed locally from its own AST and Git history — no uploading code to a SaaS dashboard, no dependency on a paid platform like CodeScene or SonarQube.

```text
$ cargo sig -a

[cargo-sig] Cargo SIG - Running check...
[cargo-sig] ⏳ Generating coverage data via cargo-llvm-cov... 100%

Summary:
Total Functions: 76
Volume > 15 lines: 2
Interface > 4 params: 0
Complexity > 5 branches: 1
Code Duplication: 1.3%

Component Balance:
  All components are balanced. ✅

Module Coupling:
  40 external dependencies ignored. ℹ️
  Fan-Out is healthy across all modules. ✅
  No Circular Dependencies. ✅

Risk Profile:
Moderate Risk: 7.3%
High Risk: 0.0%
Very High Risk: 0.0%

─────────────────────────────────────
Maintainability Rating:
  Code Health:   ★★★★★★★ (7 / 7)
  Test Coverage: ★★★★★★★ (7 / 7) [96.4% weighted]
  System Volume: ★★★★★★★ (7 / 7) [Total: 703 func LOC]
  ──────────────────────────────
  Final Score:   ★★★★★★★ (7 / 7)
```

*(Actual output running against its own v0.7.1 repository)*

---

## Table of Contents

- [Why cargo-sig](#why-cargo-sig)
- [Requirements](#requirements)
- [Installation](#installation)
- [Usage](#usage)
- [What Gets Measured](#what-gets-measured)
- [CI Integration](#ci-integration)
- [Exit Codes](#exit-codes)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)

---

## Why cargo-sig

Static analysis for Rust today means either a fast linter (`clippy`) that checks style and correctness, or a paid SaaS platform that computes maintainability trends from your Git history. There is no local, native middle ground.

`cargo-sig` gives you:

- **A published, non-arbitrary model.** Ratings are computed against SIG's [10 Guidelines for Maintainable Software](https://www.softwareimprovementgroup.com/) — the same guidelines behind Better Code Hub — not an invented scoring formula.
- **Churn-aware prioritization.** Complexity alone doesn't tell you what to refactor first. Crossing it with how often a file actually changes (via Git history) surfaces the handful of files doing the most damage — the "hotspots" from Adam Tornhill's *Your Code as a Crime Scene*.
- **Native AST parsing.** Built directly on [`tree-sitter`](https://tree-sitter.github.io/tree-sitter/) with the official Rust grammar — no external metrics service, no Node.js runtime. Protected against stack overflows with fuzz-proof AST recursion bounds.
- **High-Performance Architecture.** Uses `rayon` to parallelize file ingestion and AST parsing across all available CPU cores, making it ultra-fast even on massive monorepos.
- **Enterprise-Grade Quality.** The engine itself enforces rigorous code quality standards, compiling cleanly under `#![warn(clippy::pedantic)]` and strict SIG guidelines.
- **CI-ready by design.** Fail a build the moment a project's rating drops below a threshold you set.

---

## Requirements

| Requirement | Minimum version |
|---|---|
| Rust toolchain | 1.85.0 (required by the 2024 edition) |
| Cargo | ships with Rust |
| Operating system | Linux · macOS · Windows |
| Git | required for the Churn analysis module |

**Core dependencies:** `tree-sitter` + `tree-sitter-rust` (AST) · `owo-colors` (terminal output) · `anyhow` (errors). *We aggressively hand-roll features like CLI parsing (`std::env::args`) to maintain an ultra-minimal dependency tree.*

---

## Installation

To install `cargo-sig` globally via crates.io:

```bash
cargo install cargo-sig
```

**From source:**

```bash
cargo install --git https://github.com/Vicente-Alejandro/Cargo-Software-Improvement-Group-Analyzer --tag v0.6.1
cd Cargo-Software-Improvement-Group-Analyzer
cargo install --path .
```

After installation, run it as a Cargo subcommand in any project:

```bash
cargo sig
```

---

## Usage

From the root of any Rust project:

```bash
cargo sig
```

### Command-line Arguments

| Flag | Description | Status |
|---|---|---|
| `-a`, `--auto-cov` | Enable automatic test coverage generation using `cargo-llvm-cov` | Completed |
| `--fail-below <1-7>` | Exit non-zero if the rating drops below the threshold (for CI gates) | Completed |
| `--format json\|html` | Export detailed structured reports instead of the terminal summary | Planned |
| `-h`, `--help` | Print the CLI help overview | Completed |

Example — using it as a CI quality gate:

```bash
cargo sig --fail-below 3
```

### Coverage Ingestion

`cargo-sig` can dynamically cross-reference your architectural hotspots with your test coverage. 

By running `cargo sig -a` (or `--auto-cov`), the tool will automatically invoke `cargo-llvm-cov` in the background, rendering an asymptotic progress bar until LCOV data is successfully generated. It then parses this data and maps it against Git Churn to calculate your **Test Coverage** and **Maintainability Rating**.

If you run `cargo sig` without the `-a` flag, it runs an ultra-fast static check, ignoring coverage. If you try to run with `-a` but `cargo-llvm-cov` is not installed, it degrades gracefully and kindly advises: `N/A (Run 'cargo install cargo-llvm-cov' to enable)`.

---

## What Gets Measured

| Guideline / Module | What it checks | Status |
|---|---|---|
| SIG Guideline 1 — Short Units | Flags units longer than 15 lines of code | Completed |
| SIG Guideline 2 — Simple Units | Cyclomatic complexity (branch points ≤ 4 per unit) | Completed |
| SIG Guideline 3 — Write Code Once | Duplication percentage | Completed |
| SIG Guideline 4 — Small Interfaces | Flags signatures with more than 4 parameters | Completed |
| SIG Guideline 9 — Balance Components | Checks if any module dominates >50% of the codebase | Completed |
| Churn × Coverage | Cross-references `git log` and `coverage.lcov` with bad quality files | Completed |
| Scoring Engine | Normalizes all of the above into a 1–7 star rating | Completed |

See [ROADMAP.md](./ROADMAP.md) for the full technical breakdown of each phase, the crates each module depends on, and open architectural risks.

---

## CI Integration

`cargo-sig` exits with a non-zero code when a project's rating falls below the configured threshold, making it compatible with any CI system that checks exit codes.

### GitHub Actions

```yaml
# .github/workflows/sig.yml
name: Maintainability Gate

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  cargo-sig:
    name: cargo-sig
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0   # full history — required by the Churn module

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Install cargo-sig
        run: cargo install cargo-sig

      - name: Run maintainability gate
        run: cargo sig --fail-below 3
```

> **Note:** the Churn module needs full Git history, not a shallow clone — set `fetch-depth: 0` in `actions/checkout`.

---

## Exit Codes

| Code | Meaning |
|---|---|
| `0` | Rating met or exceeded the configured threshold |
| `1` | Rating fell below the threshold, or a check failed |
| `2` | Internal error — could not parse the project or read its Git history |

---

## Roadmap

This is an active, phased build. See [ROADMAP.md](./ROADMAP.md) for the full plan — theoretical foundations, verified dependency choices, an architecture diagram, and phase-by-phase milestones from the current AST engine through the complete 1–7 star scoring model.

---

## Contributing

This project is currently developed as part of a personal engineering portfolio. Issues and discussion are welcome; contribution guidelines will be published once the core analysis engine (Phases 0–2 of the roadmap) is stable.

---

## License

MIT License. See [LICENSE](./LICENSE) for details.