<div align="center">

# Cargo Software Improvement Group Analyzer

**Local maintainability rating for Rust projects — SIG methodology, zero SaaS.**

[![Crates.io](https://img.shields.io/crates/v/cargo-sig.svg?style=flat-square&color=orange)](https://crates.io/crates/cargo-sig)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](./LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)

`tree-sitter` AST analysis · SIG 10-guideline checks · Churn × Coverage hotspots · Markdown, HTML & PDF reports — in one command, with a 1–7 star rating.

**Current version:** `1.2.5`

</div>

---

## Overview

`cargo-sig` is a zero-configuration Cargo subcommand that rates the maintainability of a Rust codebase using the [Software Improvement Group (SIG)](https://www.softwareimprovementgroup.com/) model — the standard behind ISO/IEC 25010 software quality benchmarks and Better Code Hub — cross-referenced with a Churn × Coverage hotspot analysis inspired by Michael Feathers' *Working Effectively with Legacy Code* and Adam Tornhill's *Your Code as a Crime Scene*.

It computes a **single, standardized maintainability score (1–7 stars)** directly on your local machine using your source AST and Git history — completely offline, with zero SaaS telemetry and no external cloud services.

```text
$ cargo sig -a

[cargo-sig] Cargo SIG - Running check...
[cargo-sig] Generating coverage data via cargo-llvm-cov... 100%

Summary:
Total Functions: 180
Volume > 15 lines: 0
Interface > 4 params: 0
Complexity > 5 branches: 0
Code Duplication: 1.1%

Component Balance:
  All components are balanced. [OK]

Module Coupling:
  64 external dependencies ignored.
  Fan-Out is healthy across all modules. [OK]
  No Circular Dependencies. [OK]

Risk Profile:
Moderate Risk: 0.0%
High Risk: 0.0%
Very High Risk: 0.0%

─────────────────────────────────────
Maintainability Rating:
  Code Health:   ★★★★★★★ (7 / 7)
  Test Coverage: ★★★★★★★ (7 / 7) [96.0% weighted]
  System Volume: ★★★★★★★ (7 / 7) [Total: 1630 func LOC]
  ──────────────────────────────
  Final Score:   ★★★★★★★ (7 / 7)

Tip: Run 'cargo sig -r' (Markdown), 'cargo sig -w' (HTML), or 'cargo sig -p' (PDF) for full reports. 'cargo sig -h' for help.
```

*(Actual output running against `cargo-sig`'s own repository)*

---

## Table of Contents

- [Why cargo-sig](#why-cargo-sig)
- [Requirements](#requirements)
- [Installation](#installation)
- [Usage](#usage)
  - [Command-Line Flags](#command-line-flags)
  - [Coverage Ingestion](#coverage-ingestion)
- [Reporting Ecosystem](#reporting-ecosystem)
  - [Terminal Summary](#terminal-summary)
  - [Markdown Report](#markdown-report)
  - [Interactive Offline HTML Dashboard](#interactive-offline-html-dashboard)
  - [Executive PDF Audit Report](#executive-pdf-audit-report)
  - [Machine-Readable JSON Export](#machine-readable-json-export)
  - [Automatic Workspace `.gitignore` Safety](#automatic-workspace-gitignore-safety)
- [What Gets Measured (SIG Quality Model)](#what-gets-measured-sig-quality-model)
- [CI/CD Quality Gates](#cicd-quality-gates)
- [Exit Codes](#exit-codes)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)

---

## Why cargo-sig

Static analysis for Rust traditionally means either a fast compiler linter (`clippy`) focusing on style and idioms, or heavy enterprise SaaS tools requiring remote code uploads and recurring subscriptions. `cargo-sig` fills the gap with a native, local, high-precision analyzer:

- **Published, Industry-Standard Quality Model:** Ratings map directly to SIG's published guidelines and ISO/IEC 25010 maintainability criteria.
- **Git Churn Prioritization:** Cross-references structural complexity with Git commit frequency, highlighting genuine architectural hotspots rather than harmless legacy code.
- **Native `tree-sitter` AST Parsing:** Analyzes exact concrete syntax trees with recursion bounds and zero C/Node.js runtime overhead.
- **Parallel AST Analysis:** Powered by `rayon` for instant multi-threaded analysis across large repositories.
- **Rich Multi-Format Reporting:** Generates comprehensive Markdown, interactive dark-themed HTML dashboards, and print-ready PDF executive reports locally.
- **Zero Configuration & Minimal Footprint:** Works instantly out-of-the-box in any Rust project with hand-rolled argument parsing and minimal dependencies.

---

## Requirements

| Requirement | Minimum Version | Note |
|---|---|---|
| Rust Toolchain | 1.85.0+ | Compatible with Rust 2024 edition |
| Cargo | Ships with Rust | Subcommand integration |
| Git | 2.0+ | Required for Git Churn & Hotspot analysis |
| `cargo-llvm-cov` *(optional)* | Latest | Required only when using `-a` / `--auto-cov` |
| Chrome / Edge / Chromium *(optional)* | Any recent | Required only for automated PDF export (`-p` / `--pdf`) |

---

## Installation

### From crates.io

```bash
cargo install cargo-sig
```

### From Source

```bash
git clone https://github.com/Vicente-Alejandro/Cargo-Software-Improvement-Group-Analyzer.git
cd Cargo-Software-Improvement-Group-Analyzer
cargo install --path .
```

After installation, invoke it inside any Rust project:

```bash
cargo sig
```

---

## Usage

### Command-Line Flags

| Flag | Long Form | Description | Output Target |
|---|---|---|---|
| `-a` | `--auto-cov` | Automatically generate and ingest test coverage data via `cargo-llvm-cov` | stdout |
| `-r` | `--report` | Generate full Markdown audit report | `tools/cargo-sig/SIG_REPORT.md` |
| `-w` | `--html`, `--web` | Generate interactive, standalone offline HTML dashboard | `tools/cargo-sig/SIG_REPORT.html` |
| `-p` | `--pdf` | Generate standalone executive PDF audit report via headless browser | `tools/cargo-sig/SIG_REPORT.pdf` |
| | `--fail-below <1-7>` | Set minimum maintainability score threshold for CI/CD quality gates | Non-zero exit code |
| | `--format json` | Export detailed machine-readable JSON structure | stdout |
| `-h` | `--help` | Display command-line options and usage summary | stdout |
| `-V` | `--version` | Display current `cargo-sig` version | stdout |

Multiple reporting flags can be combined in a single execution:

```bash
cargo sig -a -r -w -p
```

### Coverage Ingestion

Running `cargo sig -a` (or `--auto-cov`) invokes `cargo-llvm-cov` with an asymptotic terminal progress indicator, generating and parsing LCOV coverage metrics without requiring manual file handling.

If `cargo-llvm-cov` is not installed on your system, `cargo-sig` degrades gracefully:
```text
Test Coverage: N/A (cargo-llvm-cov not installed - Run 'cargo install cargo-llvm-cov')
```

---

## Reporting Ecosystem

All generated reports are stored cleanly under `tools/cargo-sig/` to avoid polluting your workspace root.

```text
my-rust-project/
├── tools/
│   └── cargo-sig/
│       ├── SIG_REPORT.md    <-- Full Markdown breakdown
│       ├── SIG_REPORT.html  <-- Interactive visual dashboard
│       └── SIG_REPORT.pdf   <-- Print-ready executive audit
└── .gitignore               <-- Automatically protected
```

### Terminal Summary
Provides an instant high-level overview featuring:
- Function metric violation totals (Volume, Interface, Complexity, Duplication)
- Component balance and module coupling health
- Proportional risk profile distribution (Moderate, High, Very High)
- 1–7 star rating breakdown (Code Health, Test Coverage, System Volume, Final Score)

### Markdown Report
Generates `tools/cargo-sig/SIG_REPORT.md` formatted with GitHub-flavored markdown tables, code blocks, and clear violation callouts for inclusion in pull requests or project documentation.

### Interactive Offline HTML Dashboard
Generates `tools/cargo-sig/SIG_REPORT.html` featuring:
- **Executive Glassmorphic Theme:** Dark-palette UI with responsive typography (`Inter` + `JetBrains Mono`).
- **Dynamic SVG Score Gauge:** Radial SVG score visualization with semantic status coloring.
- **Interactive Tab Navigation:** Switch instantly between *Overview*, *Violations*, *Hotspots Matrix*, *Duplication Spans*, and *Architecture*.
- **Inline Source Code Expander:** Click **View Code** to inspect exact source snippets with syntax-highlighted line numbers without leaving the browser.
- **100% Offline & Standalone:** Zero external CDN dependencies, web fonts, or tracking scripts.

### Executive PDF Audit Report
Generates `tools/cargo-sig/SIG_REPORT.pdf` using local headless browser printing (Chrome, Chromium, Edge, or Brave):
- **ISO/IEC 25010 Compliance Header:** Formal audit report styling.
- **Optimized Print Pagination:** Automated `@media print` layout with dedicated section page breaks.
- **Hidden Interactive Elements:** Suppresses interactive tabs, buttons, and action columns for a publication-ready physical or digital PDF document.

### Machine-Readable JSON Export
Export full analysis data via `--format json` for integration with custom dashboards, telemetry collectors, or bespoke CI tooling:

```bash
cargo sig --format json > sig-metrics.json
```

### Automatic Workspace `.gitignore` Safety
When running report generation commands (`-r`, `-w`, `-p`), `cargo-sig` verifies whether `tools/cargo-sig/` is listed in your project's `.gitignore`. If missing, it offers an interactive prompt or automatically adds the entry, preventing generated artifacts from polluting your Git repository.

---

## What Gets Measured (SIG Quality Model)

| Dimension / Guideline | Target Standard | Metric & Diagnostic |
|---|---|---|
| **SIG 1: Short Units of Code** | LOC ≤ 15 lines | Flags long functions that should be decomposed into smaller units. |
| **SIG 2: Simple Units of Code** | CC ≤ 5 branch points | Evaluates cyclomatic complexity (`if`, `match`, `while`, `for`, `?`). |
| **SIG 3: Write Code Once** | Duplication ≤ 3.0% | AST block hashing to detect copied and pasted code fragments. |
| **SIG 4: Keep Interfaces Small** | Params ≤ 4 | Flags functions with excessive parameter counts. |
| **SIG 5 & 6: Module Coupling** | Fan-Out ≤ 7, No Cycles | Inspects `use` dependencies and identifies circular dependency chains. |
| **SIG 7: Balance Components** | ≤ 50% LOC per module | Verifies that no single top-level directory dominates the codebase. |
| **SIG 8: System Volume** | Scaled LOC limits | Grades total functional lines of code against maintainability volume scales. |
| **SIG 9: Automated Test Coverage** | Weighted Coverage % | Computes churn-weighted test coverage percentage across the codebase. |
| **Churn × Coverage Hotspots** | Risk Points × Commits | Surfaces high-complexity, frequently changed files with low test coverage. |

---

## CI/CD Quality Gates

Use `--fail-below <1-7>` to enforce minimum maintainability scores in CI pipelines.

### GitHub Actions Workflow

```yaml
# .github/workflows/maintainability.yml
name: Maintainability Quality Gate

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  sig-gate:
    name: SIG Maintainability Gate
    runs-on: ubuntu-latest

    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
        with:
          fetch-depth: 0 # Full history required for Git Churn analysis

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Install cargo-sig
        run: cargo install cargo-sig

      - name: Run Maintainability Gate
        run: cargo sig --fail-below 4 -r -w
```

---

## Exit Codes

| Code | Status | Meaning |
|---|---|---|
| `0` | **Success** | Analysis completed successfully and maintainability rating met the threshold. |
| `1` | **Quality Failure** | Rating fell below `--fail-below` threshold, or analysis check failed. |
| `2` | **System Error** | I/O error, invalid arguments, or failure reading workspace / Git history. |

---

## Roadmap

See [ROADMAP.md](./ROADMAP.md) for the complete version history and upcoming milestones, including historical tracking (`.sig_history.md`) and delta trend sparklines in **v1.3.0**.

---

## Contributing

Contributions, feedback, and issue reports are welcome! Please check the [issue tracker](https://github.com/Vicente-Alejandro/Cargo-Software-Improvement-Group-Analyzer/issues) or open a discussion on architecture enhancements.

---

## License

This project is licensed under the [MIT License](./LICENSE).