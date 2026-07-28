# cargo-sig ⭐️

**Cargo Software Improvement Group Analyzer**

`cargo-sig` is an ultra-minimal, high-performance static analysis tool for Rust projects. It evaluates your codebase based on the [Software Improvement Group (SIG)](https://www.softwareimprovementgroup.com/) maintainability model and translates the metrics into a **1 to 7 star rating**.

Built with **Data-Oriented Design** and **Fearless Concurrency** in mind, `cargo-sig` compiles in seconds and runs in milliseconds. We deliberately eliminated hundreds of transitive dependencies (no `clap`, no async runtimes, no web servers) to keep the footprint as small as possible.

---

## Features

- **Zero-Configuration:** No setup required. Just run `cargo sig` in any Rust project directory.
- **Deep Modules & Native Parsing:** Uses `tree-sitter` for robust, native Abstract Syntax Tree (AST) traversal.
- **Data-Oriented Metrics:** Collects metrics in flat arrays for cache-coherent performance.
- **SIG 10 Guidelines Evaluation:**
  - **Rule 1 (Short Units):** Flags functions exceeding 15 lines of code.
  - **Rule 4 (Small Interfaces):** Flags function signatures with more than 4 parameters.
  - *(More rules coming in future phases)*
- **Quality Gates:** Built-in support for CI/CD pipelines to fail builds if the codebase drops below a desired star rating.

## Installation

Currently, `cargo-sig` is in active development (v0.2.1). You can install it locally from source:

```bash
git clone https://github.com/Vicente-Alejandro/Cargo-SIG-Software-Improvement-Group-Analyzer-.git
cd cargo-sig
cargo install --path .
```

*(Thanks to our ultra-minimal architecture, compilation takes < 20 seconds from scratch!)*

## Usage

Navigate to any Rust project and run:

```bash
cargo sig
```

### Advanced Options

You can trigger a failure (exit code `1`) if the project does not meet your quality standards. This is perfect for GitHub Actions or CI pipelines:

```bash
cargo sig --fail-below 4
```

## Architecture Roadmap

The project is actively being developed. See [ROADMAP.md](./ROADMAP.md) for the detailed implementation phases covering:
- Concurrency via `rayon`.
- Cyclomatic Complexity (Rule 2).
- Hotspot Analysis (Churn vs Coverage).
- Component Balance (Module Coupling).

## License

MIT License. See [LICENSE](./LICENSE) for details.
