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

**Scripting on the surface. Compiled at its core. | AI-Augmented Data Pipeline Language**

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Language: .xzz](https://img.shields.io/badge/Language-.xzz-orange.svg)]()
[![Backend: Rust + Polars](https://img.shields.io/badge/Backend-Rust%20%2B%20Polars-red.svg)]()
[![Crates.io](https://img.shields.io/badge/crates.io-x1zz-orange)](https://crates.io/crates/x1zz)
[![Status: v0.2.0](https://img.shields.io/badge/Status-v0.2.0-green.svg)]()

[한국어 README](README_kr.md)

</div>

---

## Installation

```bash
cargo install x1zz
```

> Requires Rust 1.85+. Install Rust at [rustup.rs](https://rustup.rs)

Or build from source:

```bash
git clone https://github.com/x1zzdev/x1zzLang.git
cd x1zzLang
cargo build --release
```

---

## Quick Start

```bash
x1zz new demo
cd demo
x1zz run example.xzz
```

Expected output:

```
📊 [x1zz Execution Result: 'result' (Top 5 Rows)]
────────────────────────────────────────────────────────────
shape: (7, 4)
┌───────────┬──────┬──────┬────────────┐
│ station   ┆ pm10 ┆ pm25 ┆ date       │
│ ---       ┆ ---  ┆ ---  ┆ ---        │
│ str       ┆ f64  ┆ f64  ┆ str        │
╞═══════════╪══════╪══════╪════════════╡
│ Songpa    ┆ 68.1 ┆ 36.4 ┆ 2026-01-10 │
│ Dobong    ┆ 55.9 ┆ 29.1 ┆ 2026-01-07 │
│ Gangseo   ┆ 52.3 ┆ 28.4 ┆ 2026-01-02 │
│ ...       ┆ ...  ┆ ...  ┆ ...        │
└───────────┴──────┴──────┴────────────┘
```

Export to CSV:

```bash
x1zz run example.xzz --output result.csv
```

---

## CLI

```bash
x1zz new   <project-name>                     # create new project
x1zz run   <file.xzz>                         # execute pipeline
x1zz run   <file.xzz> --output result.csv     # execute + export CSV
x1zz run   <file.xzz> --verbose               # show token stream + AST
x1zz check <file.xzz>                         # static analysis
x1zz emit  rust <file.xzz>                    # emit transpiled Rust source
x1zz import <data.csv>                        # auto-generate type + load statement
```

---

## Examples

### 1. CSV Load + Filter + Export

```xzz
type AirQuality = {
    station: string,
    pm10: float,
    pm25: float,
    date: string,
}

v data = load("data/sample.csv") :: AirQuality

v result = data
    |> filter(col("pm10") > 40.0)
    |> orderBy("pm10", desc: true)
```

```bash
x1zz run example.xzz --output result.csv
```

---

### 2. GroupBy + Mean Aggregation

```xzz
type SalesData = {
    region: string,
    product: string,
    revenue: float,
}

v data = load("data/sales.csv") :: SalesData

v by_region = data
    |> groupBy("region")
    |> mean("revenue")
    |> orderBy("revenue", desc: true)
```

---

### 3. Full Pipeline (Filter → GroupBy → Sum → Take)

```xzz
type WelfareSchema = {
    region:     string,
    population: int,
    income:     Option<float>,
    support:    bool,
}

v data = load("data/welfare_2026.csv") :: WelfareSchema

v blind_spots = data
    |> dropNull("income")
    |> filter(col("income") < 1_200_000)
    |> filter(col("support") == false)
    |> groupBy("region")
    |> count("population")
    |> orderBy("population", desc: true)
    |> take(10)
```

---

### 4. Seoul Air Quality Pipeline (included in repo)

```bash
x1zz run examples/poc_script.xzz
```

Uses real Seoul air quality CSV data included in `examples/`.

---

## Benchmark

![x1zzLang Benchmark](benches/x1zzLang_benchmark2.png)

> **Performance Snapshot:** On our benchmarked 3.4 million-row workload, `x1zzLang` achieved a **3.84× speedup** over an equivalent Pandas pipeline by compiling data pipelines into optimized Polars LazyFrame execution plans with automatic parallelization. *(For a visual workflow interface, see the [x1zzETL Visual IDE](https://github.com/x1zzdev/x1zzLang-visual-ide).)*

---

## Why x1zzLang

Every year, thousands of public datasets are released in South Korea —  
welfare gaps, air quality distributions, transportation blind spots.  
The data already exists.

**Then why do the problems persist?**

Not because the data is missing. Because **most people cannot analyze it.**

The real barriers to public data analysis today:

| Barrier | Problem |
|---------|---------|
| Library prerequisite | `pandas`, `numpy`, `matplotlib` — three libraries before you can start |
| Runtime type errors | Column name typos and type mismatches surface only at execution |
| Environment friction | `pip install`, version conflicts, virtual environment setup |
| Unpredictable output | No way to reason about results before running the code |

x1zzLang removes these barriers **at the language design level**.  
`filter`, `groupBy`, and `mean` are not library calls — they are **native syntax**.  
Welfare officers, environmental researchers, and policy analysts should be able to work with data directly.

---

```xzz
// This is all it takes. No imports. No main(). No boilerplate.
type WelfareSchema = {
  region:     string,
  population: int,
  income:     Option<float>,   // nullable — expected in public datasets
  support:    bool,
}

v data = load("welfare_2026.csv") :: WelfareSchema

v blind_spots = data
  |> filter(col("income") < 1_200_000)
  |> filter(col("support") == false)
  |> groupBy("region")
  |> count("population")
  |> orderBy("population", desc: true)
```

> *Execution flows from the top of the file. The language is the analysis.*

---

## Compiler Architecture

```
x1zz-compiler
│
├── Lexer          — tokenization of .xzz source
├── Token          — token type definitions
├── Parser         — recursive descent parser
├── AST            — abstract syntax tree nodes
├── Type Checker   — schema validation, type inference
├── Code Generator — Rust transpiler (LazyFrame emission)
└── Error          — structured compile-time diagnostics
```

**Pipeline:**

```
.xzz source
     │
     ▼
  Lexer  →  Token stream
     │
     ▼
  Parser  →  AST
     │
     ▼
  Type Checker (Schema + pipeline type inference)
     │
     ▼
  Code Generator  →  Rust source
     │
     ▼
  rustc + Polars LazyFrame  →  Native binary
```

---

## Core Pillars

### 1. Data-Native Syntax

> *"If analysis operations live in a library, the language doesn't understand analysis."*

In x1zzLang, `filter`, `groupBy`, `sum`, `mean`, `orderBy` are **language grammar**.  
Each pipeline step compiles to a Polars LazyFrame operation node, executed over Apache Arrow's zero-copy memory layout.

```xzz
// The pipeline is not syntactic sugar.
// Each |> step maps to a Polars LazyFrame node.
v result = data
  |> filter(col("price") > 100)      // .filter(col("price").gt(100))
  |> groupBy("region")               // .group_by(["region"])
  |> mean("price")                   // .agg([col("price").mean()])
  |> orderBy("price", desc: true)    // .sort("price", descending: true)
  |> take(10)                        // .limit(10)  ← collect() deferred to final step
```

`.xzz` source is transpiled to Rust and compiled to a native binary:

```
.xzz  →  Rust (transpile)  →  Native Binary (Polars LazyFrame)
```

---

### 2. Experimental AI-Augmented Compilation — Neural Query Planner (NQP)

> *"If GitHub Copilot completes code, x1zz-Copilot is part of the compiler that understands execution."*

The AI layer in x1zzLang is not a code generation assistant.  
It **predicts the state of data across pipeline steps before execution begins.**

The Neural Query Planner (NQP) analyzes pipeline structure and data distribution to reason about results ahead of time.

```
$ x1zz run welfare_analysis.xzz --predict

╔══════════════════════════════════════════════════════════════╗
║  x1zz Neural Query Planner — State Prediction               ║
╠══════════════════════════════════════════════════════════════╣
║  Pipeline: welfare_analysis.xzz                             ║
║  ─────────────────────────────────────────────────────────  ║
║  Step 1  filter(income < 1_200_000)                         ║
║          rows_before : 142,300                              ║
║          rows_after  : ~38,400  (est. 27.0% selectivity)    ║
║                                                             ║
║  Step 2  filter(support == false)                           ║
║          rows_after  : ~12,100  (est. 31.5% selectivity)    ║
║                                                             ║
║  Step 3  groupBy("region") |> count("population")           ║
║          groups      : ~17 regions                          ║
║          top_region  : "Gyeonggi North"  ~2,340 (est.)      ║
║                                                             ║
║  Confidence: 87.3%  |  Model: [internal-model]              ║
╚══════════════════════════════════════════════════════════════╝

  Proceed? [y/N]
```

These predictions are not execution results — they are pre-execution inferences derived from pipeline structure and data distribution.  
They allow users to validate pipeline logic before committing to a full run.

---

### 3. Safe by Default

> *"The moment data is loaded, it is already validated."*

The `::` Safe-Load operator and schema-aware type inference eliminate an entire class of runtime errors before execution.

```xzz
// Declare the schema first.
type SalesSchema = {
  date:     date,
  price:    float,
  region:   string,
  quantity: int,
  discount: Option<float>,   // nullable — some records have no discount
}

// :: verifies column presence and schema compatibility before execution.
v data = load("sales_2026.csv") :: SalesSchema
```

Column name typos and type mismatches are caught before execution:

```
[SchemaError] at analysis.xzz:3:33
─────────────────────────────────────────────
Cause   : Column referenced in filter() does not exist in schema.
Detail  : column 'pric' not found in SalesSchema
Available: date, price, region, quantity, discount
→ Did you mean: col("price")
```

---

## SDE — Synthetic Data Engine

> *"Garbage data produces garbage predictions."*

The NQP's prediction quality depends entirely on the quality of its training data.  
Public datasets are riddled with missing values, outliers, and inconsistent formatting.

x1zzLang adopts an **SDE-First** methodology: before the NQP can make reliable predictions, it must be trained on data that faithfully represents the statistical reality of public datasets — without exposing any real personal information.

The Synthetic Data Engine generates large-scale, high-fidelity training datasets that preserve the statistical properties of real data while introducing controlled noise, edge cases, and distribution shifts. This is how the NQP learns to reason about real-world pipelines.

Privacy protection is not a feature — it is a structural guarantee of this approach.

---

## Python vs. x1zzLang

**Scenario**: Identify welfare blind spots — regions where residents fall below the income threshold but receive no support.

### Python (pandas)

```python
import pandas as pd
import numpy as np

df = pd.read_csv("welfare_2026.csv")  # no type validation

df_filtered = df[df['income'] < 1200000]
df_no_support = df_filtered[df_filtered['support'] == False]

df_no_support = df_no_support.dropna(subset=['income'])  # manual null handling

result = df_no_support.groupby('region')['population'].count()
result = result.sort_values(ascending=False)

print(result.head(10))
```

*Three libraries, manual null handling, runtime type errors, no pre-execution insight.*

### x1zzLang

```xzz
type WelfareSchema = {
  region:     string,
  population: int,
  income:     Option<float>,
  support:    bool,
}

v data = load("welfare_2026.csv") :: WelfareSchema

v blind_spots = data
  |> dropNull("income")
  |> filter(col("income") < 1_200_000)
  |> filter(col("support") == false)
  |> groupBy("region")
  |> count("population")
  |> orderBy("population", desc: true)
  |> take(10)

blind_spots |> plot.bar(x: "region", y: "population")
```

| | Python (pandas) | x1zzLang |
|--|-----------------|----------|
| Lines of code | ~15 | ~9 (40% reduction) |
| Type validation | Runtime | Compile time |
| Null handling | Manual | Enforced via `Option<T>` |
| Pre-execution prediction | No | NQP inference |
| Library dependencies | pandas, numpy, matplotlib | None (built into the language) |
| Execution engine | Python GIL | Rust + Polars LazyFrame |

---

## Roadmap

| Phase | Goal | Key Components | Status |
|-------|------|----------------|--------|
| **Phase 1** — Language Core | Complete language foundation | Lexer, Parser, AST, Type System, Pipeline Operator (`\|>`), Safe-Load (`::`) | Complete |
| **Phase 2** — Execution Layer | Full execution environment | Polars integration, incremental compilation, CLI (`x1zz run/check/fmt/emit`) | In Progress |
| **Phase 3** — Prediction Layer | AI inference layer | SDE, NQP model training, State Prediction | In Progress |
| **Phase 4** — Copilot OS | Natural language interface | Natural language → pipeline, MCP server, x1zz-Copilot integration | Vision |

---

## Release Notes

### v0.2.0 — MVP Release

- `x1zz new` now generates a fully runnable `example.xzz` + `data/sample.csv`
- `x1zz run --output result.csv`: CSV export added
- `cargo install x1zz` available on crates.io
- Version bump following semantic versioning

### v0.17 — Debug & Normalization Pass
- `BoolLit` support: `col("support") == false` now evaluates correctly in filter conditions
- pm10 / pm25 Float64 normalization: null-safe cast + `fill_null(0.0)` for Korean public datasets
- Debug output suppression: benchmark stdout pipe deadlock resolved
- Full `Lexer → Parser → Runtime` pipeline integrated in `x1zz run`

### v0.16 — Major Execution Layer Update (Runtime Completion Sprint)
A comprehensive sprint that implemented the full set of pipeline operators required to execute the README example pipeline end-to-end.

**New Pipeline Operators (9 additions)**

| Operator | Syntax | Polars Equivalent |
|----------|--------|-------------------|
| `groupBy` | `\|> groupBy("col")` | `.group_by(["col"])` |
| `sum` | `\|> sum("col")` | `.agg([col("col").sum()])` |
| `mean` | `\|> mean("col")` | `.agg([col("col").mean()])` |
| `min` | `\|> min("col")` | `.agg([col("col").min()])` |
| `max` | `\|> max("col")` | `.agg([col("col").max()])` |
| `orderBy` | `\|> orderBy("col", desc: true)` | `.sort(...)` |
| `take` | `\|> take(10)` | `.limit(10)` |
| `dropNull` | `\|> dropNull("col")` | `.filter(col.is_not_null())` |
| `fillNull` | `\|> fillNull("col", 0)` | `.with_columns([col.fill_null(lit(0))])` |

**Language additions**
- `col("x")` syntax in filter expressions
- `true` / `false` boolean literals
- Numeric underscore separators (`1_200_000`)

**Runtime: pending_group_by pattern**  
`groupBy` stores the column in a deferred slot; the following aggregation op (`sum`, `mean`, `min`, `max`, `count`) consumes it to emit a single `.group_by([...]).agg([...])` chain.

**Tests: 25 / 25 passed**

### v0.16 — Benchmark Suite
Production-grade benchmark orchestrator (`benches/run_benchmark.py`) with:
- 6 real-world EUC-KR Seoul air quality CSV files merged into 3 UTF-8 scale datasets (Small ≈ 2M rows, Medium ≈ 15M rows, Large ≈ 30M rows) via 10× statistical augmentation
- Wall-clock latency + RSS memory profiling at 3 ms sampling intervals
- Academic whitepaper-style HTML report with Chart.js visualisations (`benches/benchmark_report.html`)
- Comparison pipeline: `dropNull → filter → groupBy → sum → mean → orderBy → take` (identical logic across both engines)

### v0.16 — Visual IDE
x1zzLang Visual IDE successfully developed — providing a graphical editing and execution environment for `.xzz` pipelines.

---

## Current Status

| Component | Status |
|-----------|--------|
| Lexer / Parser | Implemented |
| AST / Token System | Implemented |
| Type System (Schema) | Implemented |
| Rust Transpiler (codegen) | Implemented |
| Pipeline Operator (`\|>`) | Implemented |
| Safe-Load (`::`) | Implemented |
| GroupBy / Agg Operators (sum, mean, min, max, count) | Implemented (v0.16) |
| OrderBy / Take / DropNull / FillNull | Implemented (v0.16) |
| BoolLit & col() expression support | Implemented (v0.17) |
| CSV Export (`--output`) | Implemented (v0.2.0) |
| SDE (Synthetic Data Engine) | Implemented |
| NQP State Prediction (PoC) | Implemented (dryrun) |
| Benchmark Suite (Pandas vs. x1zzLang) | Implemented (v0.16) |
| Visual IDE | Implemented (v0.16) |
| Polars LazyFrame integration | In Progress |
| Incremental Compilation | Planned (Phase 2) |
| NQP Model Training | Planned (Phase 3) |
| MCP Server | Phase 4 |

---

## Contributing

## Contributions & Notice

`x1zzLang` is an open-source project, and I welcome all forms of feedback and interest.

However, to comply with the **official regulations of the 8th Korea-CodeFair 2026 (ensuring pure self-development and research ethics)**, I am **temporarily suspending the acceptance of Pull Requests (code contributions) until the end of the finals in October 2026**. This measure is taken to prevent any ambiguity regarding authorship and to guarantee project integrity during the evaluation period.

* **Issues (Suggestions & Bug Reports):** 🟢 Always welcome! Please feel free to open an issue for feature requests, bug reports, or conceptual discussions.
* **Pull Requests (Code Contributions):** 🟡 Will be reopened after the final competition in October 2026.

Thank you for your understanding and support for `x1zzLang`.

---

## License

Apache-2.0

---

<div align="center">

**x1zzLang — 2026**

</div>
