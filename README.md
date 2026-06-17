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

**A DSL platform that lets non-experts perform data analysis without writing code.**

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Language: .xzz](https://img.shields.io/badge/Language-.xzz-orange.svg)]()
[![Backend: Polars](https://img.shields.io/badge/Backend-Polars-red.svg)]()
[![Status: v0.2.0](https://img.shields.io/badge/Status-v0.2.0-green.svg)]()

[한국어 README](README_kr.md)

</div>

---

## Visual IDE

[![x1zzLang Visual IDE](screenshot_visual_ide.png)](https://github.com/x1zzdev/x1zzLang-visual-ide)

A graphical editing and execution environment for `.xzz` pipelines.  
→ [x1zzLang Visual IDE repository](https://github.com/x1zzdev/x1zzLang-visual-ide)

---

## Why x1zzLang?

Data exists everywhere. Public datasets are published every year.

The barrier is not data availability — it is analysis accessibility.

Before a single row of data is touched, an analyst must install libraries, configure a Python environment, and memorize multiple APIs. Most people stop there — not because the problem is unsolvable, but because the tooling was not built for them.

| Barrier | Problem |
|---------|---------|
| Library prerequisite | Python / Pandas / SQL — code-first setup before any analysis |
| Runtime type errors | Type mismatches and column errors surface only at execution |
| Environment friction | Setup friction causes user drop-off before the first result |

x1zzLang replaces code-first analysis with DSL-based interaction.

---

## Python vs. x1zzLang

**Scenario:** Filter and aggregate a CSV dataset.

### Python (Pandas)

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

| | Python (Pandas) | x1zzLang |
|--|-----------------|----------|
| Library dependencies | `pandas`, `numpy` | None (built-in) |
| Type validation | Runtime | Schema declaration |
| Null handling | Manual | `Option<T>` |

---

## Features

| Feature | Description |
|---------|-------------|
| CSV loading | File ingestion |
| Filtering | Conditional filtering |
| Aggregation | Grouped statistics |
| Visualization | Result rendering |
| Compiler pipeline | DSL → IR transformation |
| Visual IDE | GUI editor |
| Runtime execution | Polars engine |

---

## Example

```xzz
type AirQuality = {
  date:    string,
  station: string,
  pm10:    Option<float>,
  pm25:    Option<float>,
}

v data = load("data.csv") :: AirQuality
  |> cast("pm10", "float")
  |> cast("pm25", "float")
  |> filter(pm10 > 50)
  |> select([date, station, pm10, pm25])
```

```bash
x1zz run analysis.xzz
```

---

## Installation

### 1. Download Release

Download the latest release from:

**[https://github.com/x1zzdev/x1zzLang/releases](https://github.com/x1zzdev/x1zzLang/releases)**

### 2. Extract

Extract the release package to a local folder.

### 3. Run

```bash
x1zz run <file>
```

### 4. Verify

```bash
x1zz --version
```

### Important Notes

- No Rust or Cargo required
- Standalone executable
- Bundled dependencies

---

## Quick Start

```bash
# 1. Download and extract the release package
# 2. Run a pipeline
x1zz run <file>
# 3. View output in the terminal
```

---

## Architecture

> ⚠️ Conceptual overview only.

```
x1zz-cli
├── x1zz-core
└── x1zz-compiler

x1zz-runner
└── IPC Bridge

x1zz-exec
└── Polars Runtime
```

---

## Benchmark

![x1zzLang Benchmark](benches/x1zzLang_benchmark2.png)

> *Benchmark: x1zzLang pipeline execution vs. equivalent Pandas pipeline.*

---

## Current Status

**Active Development**

---

## Roadmap

| Phase | Goal |
|-------|------|
| Phase 1 — Core Language | DSL syntax, type system, compiler pipeline |
| Phase 2 — Execution Layer | Full Polars integration, CLI tooling |
| Phase 3 — IDE Integration | Visual IDE, graphical pipeline editor |
| Phase 4 — AI Expansion | Natural language interface, AI-augmented analysis |

---

## Contributing

`x1zzLang` is an open-source project. Feedback and suggestions are welcome.

However, to ensure authorship integrity during the 8th Korea-CodeFair 2026 evaluation period, code contributions (Pull Requests) are temporarily paused until October 2026.

- Issues (bug reports, ideas, discussions): Always welcome
- Pull Requests: Closed until October 2026 (will reopen after the competition)

Thank you for your understanding and support for x1zzLang.

---

## License

Apache-2.0

---

<div align="center">

**x1zzLang — 2026**

</div>
