<div align="center">

```text
 ██╗  ██╗ ██╗ ███████╗███████╗██╗      █████╗ ███╗   ██╗ ██████╗ 
 ╚██╗██╔╝███║ ╚══███╔╝╚══███╔╝██║     ██╔══██╗████╗  ██║██╔════╝ 
  ╚███╔╝ ╚██║   ███╔╝   ███╔╝ ██║     ███████║██╔██╗ ██║██║  ███╗
  ██╔██╗  ██║  ███╔╝   ███╔╝  ██║     ██╔══██║██║╚██╗██║██║   ██║
 ██╔╝ ██╗ ██║ ███████╗███████╗███████╗██║  ██║██║ ╚████║╚██████╔╝
 ╚═╝  ╚═╝ ╚═╝ ╚══════╝╚══════╝╚══════╝╚═╝  ╚═╝╚═╝  ╚═══╝ ╚═════╝ 
```

# x1zzLang

**A Rust-based DSL platform exploring data analysis accessibility.**  
*Scripting on the surface. Compiled at its core.*

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Language: .xzz](https://img.shields.io/badge/Language-.xzz-orange.svg)]()
[![Backend: Polars](https://img.shields.io/badge/Backend-Polars-red.svg)]()
[![Version: v0.2.8](https://img.shields.io/badge/Version-v0.2.8-green.svg)](https://github.com/x1zzdev/x1zzLang/releases)

[한국어 README](README_kr.md)

</div>

---

## Project Overview

x1zzLang is a domain-specific language (DSL) designed to explore how data analysis tooling can be made more accessible. The language compiles `.xzz` scripts into optimized [Polars](https://github.com/pola-rs/polars) LazyFrame execution plans via a Rust-based compiler pipeline.

This project is primarily an exercise in **language design**, **compiler engineering**, and **type system research** — not a production-ready replacement for existing data analysis tools.

**What this project demonstrates:**
- A declarative pipeline DSL with a null-safe type system (`Option<T>`)
- A multi-crate Rust workspace that isolates heavy dependencies (Polars) from the CLI binary
- An auto schema inference tool (`x1zz import`) that generates type definitions from CSV files
- A Visual IDE for graphical pipeline editing

---

## Core Idea

Most data analysis workflows require environment setup before touching a single row of data: install Python, install libraries, configure a virtual environment, infer column types by hand, handle nulls explicitly.

x1zzLang explores a different approach: schema is declared upfront in the type system, null-safety is enforced at the type level, and the pipeline is expressed as a composition of named operations.

```
Type declaration → Pipeline composition → Compiled execution
```

The goal is not to replace existing tools but to investigate what a purpose-built, type-safe data pipeline language looks like and how far that design can go.

---

## Quick Example

**Scenario:** Filter and aggregate air quality data from a CSV file.

### Python (pandas)

```python
import pandas as pd

df = pd.read_csv("data.csv")
df = df[df["pm10"] > 50]
result = df.groupby("station")["pm10"].mean()
print(result)
```

*Requires library installation. Type errors surface at runtime. Null handling is manual.*

### x1zzLang

```xzz
type AirQuality = {
  station: string,
  pm10:    Option<float>,
}

v data = load("data.csv") :: AirQuality
  |> cast("pm10", "float")
  |> filter(pm10 > 50)
  |> groupBy("station")
  |> mean("pm10")
```

*No imports. Schema declared upfront. Null-safe via `Option<T>`.*

| | Python (pandas) | x1zzLang |
|--|-----------------|----------|
| Library dependencies | `pandas`, `numpy` | None (built-in) |
| Type validation | Runtime | Schema declaration time |
| Null handling | Manual NaN checks | `Option<T>` in type definition |

**From `x1zz import` to running pipeline:**

```bash
x1zz new my-project    # scaffold project + sample CSV
cd my-project
x1zz import data.csv   # auto-infer schema → write type block to main.xzz
x1zz run main.xzz      # compile + execute pipeline
```

---

## Result Preview

After running a pipeline with a `chart {}` block, x1zzLang renders the result as an HTML chart:

![x1zzLang Benchmark](benches/x1zzLang_benchmark2.png)

> *Example: pipeline execution result rendered as a bar chart. Chart output is written to an HTML file.*

---

## Visual IDE

[![x1zzLang Visual IDE](screenshot_visual_ide.png)](https://github.com/x1zzdev/x1zzLang-visual-ide)

A graphical editing and execution environment for `.xzz` pipelines.  
→ [x1zzLang Visual IDE repository](https://github.com/x1zzdev/x1zzLang-visual-ide)

---

## Features

| Feature | Description | Status |
|---------|-------------|--------|
| `x1zz run` | Compile and execute `.xzz` pipeline | Stable |
| `x1zz import` | Auto-infer CSV schema → generate type block | Stable |
| `x1zz new` | Scaffold project with sample CSV and runnable example | Stable |
| `x1zz emit rust` | Transpile `.xzz` → Rust source (Polars LazyFrame calls) | Stable |
| `x1zz check` | Static analysis via Neural Query Planner | Experimental |
| `x1zz sde` | Synthetic data generation engine integration | Preview |
| Built-in `chart {}` | Render pipeline results as bar / line / pie / scatter | Stable |
| `Option<T>` type system | Null-safe column declarations, `fillNull` operator | Stable |
| EUC-KR CSV support | Auto-detect and decode CP949-encoded Korean CSV files | Stable |
| Visual IDE | Graphical pipeline editor (separate repository) | Stable |

---

## Architecture

x1zzLang is structured as a Cargo workspace with intentional dependency isolation. The CLI binary does not link Polars or Tokio — those are isolated to the execution engine binary (`x1zz-runner` / `x1zz-exec`).

```
x1zz (CLI binary)
│  clap + indicatif + colored + csv + anyhow + encoding_rs
│  NO Polars  ·  NO Tokio
│
├── x1zz-compiler          Lexer → Parser → Codegen → Emitter
│   └── x1zz-core          Shared AST / Token / Error types (serde only)
│
└── [subprocess spawn] ──► x1zz-runner
                           │
                           └── x1zz-exec       Polars LazyFrame runtime
```

**Crate responsibilities:**

| Crate | Role | Heavy deps |
|-------|------|------------|
| `x1zz` (CLI) | Argument parsing, import, new, emit, check | None |
| `x1zz-core` | Shared AST, Token, Error types | serde only |
| `x1zz-compiler` | Lexer / Parser / Codegen / Emitter | None |
| `x1zz-exec` | Polars execution engine | **Polars, encoding_rs** |
| `x1zz-runner` | Execution binary (spawned by CLI) | via x1zz-exec |
| `x1zz-sde` | Synthetic data generation (standalone) | polars, rayon |
| `x1zz-server` | REST API server (standalone) | axum, tokio |

**Why this structure?**  
The CLI binary stays small (~2–5 MB) because it never links Polars. When `x1zz run` is called, it spawns `x1zz-runner` as a subprocess — the runner carries all the heavy Polars dependencies independently. Communication between them uses only CLI arguments (no IPC protocol).

**Binary size trade-off:**

| Binary | Approx. Size | Contains |
|--------|-------------|----------|
| `x1zz` (CLI) | ~2–5 MB | Compiler, schema inference, project scaffolding |
| `x1zz-runner` | ~30+ MB | Polars execution engine |

For more detail, see [docs/WORKSPACE.md](docs/WORKSPACE.md).

---

## Installation

### Option A — Pre-built release (recommended)

1. Download the latest release archive for your platform from [Releases](https://github.com/x1zzdev/x1zzLang/releases).

   | Platform | Archive |
   |----------|---------|
   | Windows x64 | `x1zz-<version>-windows-x64.zip` |
   | Linux x64 | `x1zz-<version>-linux-x64.tar.gz` |
   | macOS arm64 | `x1zz-<version>-macos-arm64.tar.gz` |

2. Extract the archive. You will find `x1zz` and `x1zz-runner` in the same directory.

   > **Important:** Both binaries must remain in the same directory. `x1zz run` spawns `x1zz-runner` as a subprocess — if `x1zz-runner` is missing, pipeline execution will fail.

3. Add the extracted directory to your `PATH`.

4. Verify:

   ```bash
   x1zz --help
   ```

### Option B — Build from source

Requires Rust stable toolchain.

```bash
git clone https://github.com/x1zzdev/x1zzLang.git
cd x1zzLang

# Build CLI
cargo build --release -p x1zz

# Build execution engine
cargo build --release -p x1zz-runner

# Both binaries land in target/release/
```

Place both `x1zz` and `x1zz-runner` in the same directory before use.

---

## Benchmark

![x1zzLang Benchmark](benches/x1zzLang_benchmark2.png)

The benchmark compares x1zzLang against an equivalent pandas pipeline on a 3.4M-row Seoul air quality dataset.

> x1zzLang achieved up to **3.84× faster** execution than the pandas baseline on this workload.

This performance comes primarily from the Polars LazyFrame backend, which applies query optimization before execution. The benchmark is measuring end-to-end pipeline throughput, not compiler overhead.

Benchmark source: [`benches/run_benchmark.py`](benches/run_benchmark.py) / [`benches/benchmark_pipeline.xzz`](benches/benchmark_pipeline.xzz)

---

## Roadmap

| Phase | Goal | Status |
|-------|------|--------|
| Phase 1 — Core Language | DSL syntax, type system, compiler pipeline | Complete |
| Phase 2 — Execution Layer | Polars integration, CLI tooling, chart output | Complete |
| Phase 3 — IDE Integration | Visual IDE, graphical pipeline editor | Complete |
| Phase 4 — Expanded Language | More operators, join improvements, schema evolution | In progress |
| Phase 5 — AI Expansion | Natural language query interface (NQP), AI-augmented analysis | Experimental |

---

## Contributing

x1zzLang is an open-source project. Bug reports, ideas, and discussions via GitHub Issues are always welcome.

**Note on code contributions (Pull Requests):**  
To maintain authorship integrity during the 8th Korea CodeFair 2026 evaluation period, code contributions (Pull Requests) are temporarily suspended until October 2026. PRs will reopen after the competition concludes.

- Issues (bug reports, ideas, discussion): Open
- Pull Requests: Suspended until October 2026

See [CONTRIBUTING.md](CONTRIBUTING.md) for local build instructions and contribution guidelines.

---

## License

Apache-2.0 — see [LICENSE](LICENSE) for details.

---

<div align="center">

**x1zzLang — 2026**

</div>
