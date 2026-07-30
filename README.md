<div align="center">

# Cargo Software Improvement Group Analyzer

**Local maintainability rating for Rust projects — SIG methodology, zero SaaS.**

[![Crates.io](https://img.shields.io/crates/v/cargo-sig.svg?style=flat-square&color=orange)](https://crates.io/crates/cargo-sig)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](./LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)

`tree-sitter` AST analysis · SIG 10-guideline checks · Churn × Coverage hotspots — in one command, with a 1–7 star rating.

**Current version:** `0.6.5`

</div>

---

## Overview

`cargo-sig` is a zero-configuration Cargo subcommand that rates the maintainability of a Rust codebase using the [Software Improvement Group (SIG)](https://www.softwareimprovementgroup.com/) model — the same system behind Better Code Hub — cross-referenced with a Churn × Coverage hotspot analysis inspired by Michael Feathers' *Working Effectively with Legacy Code*.

It solves a specific problem: giving any Rust project a **single, comparable maintainability score**, computed locally from its own AST and Git history — no uploading code to a SaaS dashboard, no dependency on a paid platform like CodeScene or SonarQube.

```text
$ cargo sig

[cargo-sig] Cargo SIG - Running check...
[cargo-sig] 
[cargo-sig] Summary:
[cargo-sig] Total Functions: 60
[cargo-sig] Volume > 15 lines: 1
[cargo-sig] Interface > 4 params: 4
[cargo-sig] Complexity > 5 branches: 2
[cargo-sig] Code Duplication: 1.8%
[cargo-sig] 
[cargo-sig] Component Balance:
[cargo-sig]   ✅ All components are balanced.
[cargo-sig] 
[cargo-sig] ✅ [OK] No Hotspots.
[cargo-sig] 
[cargo-sig] Risk Profile:
[cargo-sig] Moderate Risk: 8.9%
[cargo-sig] High Risk: 0.0%
[cargo-sig] Very High Risk: 0.0%
[cargo-sig] ─────────────────────────────────────
[cargo-sig] Maintainability Rating: ★★★★★★★ (7 / 7)
```

*(Illustrative output — exact formatting will stabilize as the phases in [ROADMAP.md](./ROADMAP.md) land.)*

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
- **Native AST parsing.** Built directly on [`tree-sitter`](https://tree-sitter.github.io/tree-sitter/) with the official Rust grammar — no external metrics service, no Node.js runtime.
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
| `--fail-below <N>` | Exit non-zero if the rating drops below `N` stars (1–7) — for CI quality gates | Completed |
| `--format json\|html` | Export the detailed report instead of the terminal summary | Planned |
| `hotspots` | Print only the Churn × Coverage hotspot ranking | Completed |
| `--no-color` | Disable colored terminal output (also respects `NO_COLOR`) | Planned |

Example — using it as a CI quality gate:

```bash
cargo sig --fail-below 3
```

### Advanced: Coverage Ingestion

`cargo-sig` can cross-reference your architectural hotspots with your test coverage. It does not run your tests itself (to remain ultra-fast). Instead, it reads a `coverage.lcov` file if you generate one.

1. Generate the coverage file using a tool like `cargo-llvm-cov`:
   ```bash
   cargo llvm-cov --lcov --output-path coverage.lcov
   ```
2. Run `cargo-sig`:
   ```bash
   cargo sig
   ```

If the `coverage.lcov` file is present in the root directory, the Hotspots matrix will automatically display the test coverage percentage for your most dangerous files. If the file is not found, `cargo-sig` degrades gracefully and just prints `no cov data` next to the hotspots.

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