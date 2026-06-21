# Contributing to x1zzLang

Thank you for your interest in x1zzLang.

## Current Contribution Status

> **Pull Requests are temporarily suspended.**  
> To maintain authorship integrity during the 8th Korea CodeFair 2026 evaluation period, code contributions (PRs) are paused until October 2026. They will reopen after the competition concludes.

| Contribution type | Status |
|-------------------|--------|
| Issues (bugs, ideas, discussion) | Open |
| Pull Requests | Suspended until October 2026 |

If you have feedback or find a bug, please open a GitHub Issue. All issue reports are welcome.

---

## Project Overview

x1zzLang is a Rust-based DSL compiler platform. The workspace is structured as follows:

```
x1zz-lang/
├── src/                    x1zz CLI binary (lightweight — no Polars/Tokio)
├── x1zz-core/              Shared AST / Token / Error types
├── x1zz-compiler/          Lexer, Parser, Codegen, Emitter
├── x1zz-exec/              Polars execution engine (isolated crate)
├── x1zz-runner/            Execution binary (spawned by CLI as subprocess)
├── x1zz-server/            REST API server (standalone)
├── x1zz-sde/               Synthetic data engine (standalone, git-ignored)
├── docs/                   Architecture and workspace documentation
├── benches/                Benchmark scripts and results
└── examples/               Example .xzz scripts and CSV data
```

Key constraint: **the `x1zz` CLI binary must never link Polars or Tokio.** All Polars execution is delegated to `x1zz-runner` via subprocess.

---

## Local Build

### Prerequisites

- Rust stable toolchain ([rustup.rs](https://rustup.rs))
- Git

### Build

```bash
git clone https://github.com/x1zzdev/x1zzLang.git
cd x1zzLang

# Build CLI binary only (lightweight, no Polars)
cargo build --release -p x1zz

# Build execution engine (includes Polars — takes longer)
cargo build --release -p x1zz-runner

# Build entire workspace
cargo build --release
```

Binaries are produced in `target/release/`. For `x1zz run` to work, both `x1zz` and `x1zz-runner` must be in the same directory.

### Run a pipeline

```bash
# From target/release/ (or add to PATH)
./x1zz run examples/poc_correct.xzz
```

### Verify compiler only (no execution engine needed)

```bash
./x1zz emit rust examples/poc_correct.xzz
```

---

## Issue Guidelines

When filing a GitHub Issue, please include:

**For bug reports:**
- x1zzLang version (`x1zz --version`)
- Operating system
- `.xzz` source that reproduces the issue (minimal reproduction preferred)
- Full error output

**For feature requests or discussion:**
- What problem you are trying to solve
- What behavior you would expect
- Any relevant context

---

## Code Style

- Rust: follow `rustfmt` defaults. Run `cargo fmt` before committing.
- Commit messages: use conventional commit format (`feat:`, `fix:`, `docs:`, `chore:`, etc.).
- No Polars/Tokio imports in `x1zz` (CLI) or `x1zz-compiler` crates.

---

## Architecture Constraints

The following rules must be maintained:

1. `x1zz` (CLI) dependencies must not include: `polars`, `polars-*`, `tokio`, `rayon`, `x1zz-exec`, `x1zz-runner`.
2. `x1zz-exec` is only used by `x1zz-runner` — never by the CLI directly.
3. `x1zz-compiler` must not depend on Polars (parsing and codegen only).
4. New execution logic goes into `x1zz-exec`.

See [docs/WORKSPACE.md](docs/WORKSPACE.md) for the full dependency graph.
