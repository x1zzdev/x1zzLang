# Changelog

All notable changes to x1zzLang are documented in this file.

Format: [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)  
Versioning: [Semantic Versioning](https://semver.org/)

---

## [Unreleased]

- Open source readiness pass (repository hygiene, documentation)

---

## [v0.2.8] — 2026

### Changed
- CI: removed macOS x64 release target (arm64 only)

---

## [v0.2.7] — 2026

### Fixed
- CI: removed bash-only shell command from Windows packaging step

---

## [v0.2.5 / v0.2.4] — 2026

### Fixed
- CI: stabilized multi-platform packaging and archive validation

---

## [v0.2.3] — 2026

### Fixed
- Cargo workspace configuration
- CI pipeline fixes

---

## [v0.2.2] — 2026

### Added
- GitHub Actions release pipeline (`.github/workflows/release.yml`)
- Multi-platform build matrix: Windows x64, Linux x64, macOS arm64
- Automated archive packaging and checksum generation

---

## [v0.2.1] — 2026

### Added
- Initial release pipeline
- Binary separation: `x1zz` CLI + `x1zz-runner` + `x1zz-exec`

---

## [v0.2.0] — 2026

### Added
- MVP release
- `x1zz new` — project scaffolding with sample CSV
- `x1zz import` — CSV schema auto-inference (EUC-KR/CP949 support)
- `x1zz run` — pipeline execution via `x1zz-runner` subprocess
- `x1zz emit rust` — transpile `.xzz` to Rust (Polars LazyFrame)
- `x1zz check` — experimental NQP static analysis stub
- `x1zz sde` — synthetic data engine integration stub
- Chart visualization: `chart {}` block (bar, line, pie, scatter)
- Pipeline operators: `filter`, `groupBy`, `join`, `withColumn`, `cast`, `rename`, `sort`, `select`, `mean`, `fillNull`
- `Option<T>` null-safe type system
- Dependency isolation: Polars removed from CLI binary, isolated to `x1zz-exec`
- Multi-crate workspace: `x1zz-core`, `x1zz-compiler`, `x1zz-exec`, `x1zz-runner`, `x1zz-server`
- CSV LFS migration for large example data files
- Benchmark: 3.84× speedup over pandas on 3.4M-row workload

### Architecture
- `x1zz` CLI binary: no Polars, no Tokio (~2–5 MB)
- `x1zz-runner` spawned as subprocess for pipeline execution
- `x1zz-exec` carries Polars LazyFrame runtime (~30+ MB)

---

*Earlier development history is available via `git log`.*
