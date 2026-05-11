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

**겉은 스크립트, 속은 컴파일 | AI가 컴파일러의 일부인 첫 번째 언어**

*데이터 분석 자체가 문법인 언어.*

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Language: .xzz](https://img.shields.io/badge/Language-.xzz-orange.svg)]()
[![Backend: Rust + Polars](https://img.shields.io/badge/Backend-Rust%20%2B%20Polars-red.svg)]()
[![Status: In Development](https://img.shields.io/badge/Status-In%20Development%20(2026)-yellow.svg)]()

</div>

```xzz
// 이것이 전부다. import도, main()도, 보일러플레이트도 없다.
type WelfareSchema = {
  region:     string,
  population: int,
  income:     Option<float>,   // nullable — 공공 데이터 특성상 결측 가능
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

> *파일 상단부터 실행된다. 언어가 곧 분석이다.*

---

## Why x1zzLang — 사회 문제 해결이 곧 이 언어의 존재 이유다

대한민국에는 매년 수천 건의 공공 데이터셋이 공개된다.  
복지 사각지대, 대기오염 분포, 교통 취약 지역 — 이 데이터들은 이미 존재한다.

**그런데 왜 문제는 해결되지 않는가?**

데이터가 없어서가 아니다. **분석할 수 있는 사람이 없어서다.**

현재 공공 데이터 분석의 현실:

| 장벽 | 실제 문제 |
|------|-----------|
| Python 숙련도 | `pandas`, `numpy`, `matplotlib` — 최소 3개 라이브러리를 알아야 시작 가능 |
| 런타임 타입 에러 | 컬럼명 오타, 타입 불일치가 실행 중에 터진다 |
| 환경 의존성 | `pip install`, 버전 충돌, 가상환경 설정 |
| 결과 불확실성 | 실행 전까지 결과를 예측할 수 없다 |

x1zzLang은 이 장벽을 **언어 설계 수준에서** 제거한다.  
`filter`, `groupBy`, `mean`은 라이브러리 함수가 아니라 **언어 문법 그 자체**다.  
복지 담당 공무원도, 환경 연구자도, 교통 정책 입안자도 — 데이터를 직접 분석할 수 있어야 한다.

---

## Core Pillars — 세 가지 설계 원칙

### 1. Data-Native Syntax

> *"분석 연산이 라이브러리에 있으면, 언어는 분석을 모른다."*

x1zzLang에서 `filter`, `groupBy`, `sum`, `mean`, `orderBy`는 **언어 문법**이다.  
내부적으로 Polars LazyFrame 연산 그래프로 변환되어 Apache Arrow 기반 제로카피 메모리 위에서 실행된다.

```xzz
// 파이프라인은 문법 설탕이 아니다.
// 각 |> 단계는 Polars LazyFrame 노드로 변환된다.
v result = data
  |> filter(col("price") > 100)      // .filter(col("price").gt(100))
  |> groupBy("region")               // .group_by(["region"])
  |> mean("price")                   // .agg([col("price").mean()])
  |> orderBy("price", desc: true)    // .sort("price", descending: true)
  |> take(10)                       // .limit(10)  ← collect()는 최종 실행 시점
```

`.xzz` 코드는 Rust로 트랜스파일되어 네이티브 바이너리로 실행된다:

```
.xzz  →  Rust (transpile)  →  Native Binary (Polars LazyFrame)
```

---

### 2. AI-Augmented Compilation — Neural Query Planner (NQP)

> *"GitHub Copilot이 코드를 완성하는 도구라면,  
> x1zz-Copilot은 실행 결과를 이해하는 컴파일러의 일부다."*

x1zzLang의 AI 레이어는 코드를 **생성**하는 어시스턴트가 아니다.  
**코드를 실행하기 전에 데이터의 상태 변화를 예측**한다.

Neural Query Planner(NQP)는 파이프라인 구조와 데이터 분포를 분석하여  
실행 전 결과를 사전 추론한다. 기반 모델: DeepSeek Coder / Llama 4 기반 sLM.

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
║          top_region  : "경기 북부"  ~2,340 (est.)           ║
║                                                             ║
║  ⚡ Confidence: 87.3%  │  Model: x1zz-sLM-v0.3             ║
╚══════════════════════════════════════════════════════════════╝

  실행하시겠습니까? [y/N]
```

이 예측값은 실제 실행 결과가 아니라, **파이프라인 구조와 데이터 분포를 기반으로 한 사전 추론**이다.  
실행 전에 결과를 이해하고, 파이프라인의 논리적 오류를 사전에 발견할 수 있다.

---

### 3. Safe by Default — 런타임 오류의 원천 차단

> *"데이터를 불러오는 순간, 이미 검증된 상태다."*

`::` Safe-Load 연산자와 컴파일 타임 타입 추론으로 런타임 오류를 원천 차단한다.

```xzz
// 스키마를 먼저 선언한다.
type SalesSchema = {
  date:     date,
  price:    float,
  region:   string,
  quantity: int,
  discount: Option<float>,   // nullable — 할인 없는 경우 존재
}

// :: 연산자가 컬럼 존재 여부와 타입 일치를 컴파일 시점에 검증한다.
v data = load("sales_2026.csv") :: SalesSchema
```

컬럼명 오타나 타입 불일치는 **실행 전에** 잡힌다:

```
[SchemaError] at analysis.xzz:3:33
─────────────────────────────────────────────
Cause   : Column referenced in filter() does not exist in schema.
Detail  : column 'pric' not found in SalesSchema
Available: date, price, region, quantity, discount
→ Did you mean: col("price")
```

---

## SDE-First — 합성 데이터 엔진이 AI 신뢰도를 만든다

> *"쓰레기 데이터는 쓰레기 예측을 만든다."*

Neural Query Planner의 예측 신뢰도는 학습 데이터의 품질에 달려 있다.  
공공 데이터는 결측값, 이상치, 비정형 포맷으로 가득하다.

x1zzLang은 **SDE-First(Synthetic Data Engine First)** 방법론을 채택한다:

```
실제 공공 데이터 샘플
        │
        ▼
  SDE (Rust 기반)
  ┌─────────────────────────────────────┐
  │  • 통계 분포 보존 합성 데이터 생성  │
  │  • 결측 패턴 재현                   │
  │  • 이상치 시뮬레이션                │
  │  • Apache Arrow 포맷 직렬화         │
  └─────────────────────────────────────┘
        │
        ▼
  NQP 학습 데이터셋 (고품질, 대규모)
        │
        ▼
  신뢰할 수 있는 State Prediction
```

SDE는 Rust로 구현되어 **메모리 안전성과 생성 속도**를 동시에 보장한다.  
실제 개인정보가 포함된 데이터 없이도 NQP를 학습시킬 수 있다 — **프라이버시 보호가 기본값**이다.

---

## Social Impact Demo — Python vs x1zzLang

**시나리오**: 복지 사각지대 분석 — 소득 기준 이하이지만 지원을 받지 못하는 지역 파악

### Python (pandas) 방식

```python
import pandas as pd
import numpy as np

# 데이터 로드 — 타입 검증 없음
df = pd.read_csv("welfare_2026.csv")

# 런타임에서야 컬럼명 오타를 알 수 있다
df_filtered = df[df['income'] < 1200000]
df_no_support = df_filtered[df_filtered['support'] == False]

# NaN 처리를 별도로 해야 한다
df_no_support = df_no_support.dropna(subset=['income'])

# 집계
result = df_no_support.groupby('region')['population'].count()
result = result.sort_values(ascending=False)

print(result.head(10))
```

*라이브러리 3개, 명시적 NaN 처리, 런타임 타입 에러 위험, 실행 전 결과 예측 불가.*

### x1zzLang 방식

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

| 지표 | Python (pandas) | x1zzLang |
|------|-----------------|----------|
| 코드 라인 수 | ~15줄 | ~9줄 (**40% 감소**) |
| 타입 검증 시점 | 런타임 | **컴파일 타임** |
| NaN 처리 | 수동 명시 | `Option<T>` 타입으로 강제 |
| 실행 전 결과 예측 | ❌ 불가 | ✅ NQP 사전 추론 |
| 라이브러리 의존성 | pandas, numpy, matplotlib | **없음** (언어 내장) |
| 실행 엔진 | Python GIL | **Rust + Polars LazyFrame** |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        x1zzLang                             │
├─────────────────────────────────────────────────────────────┤
│  .xzz Source                                                │
│       │                                                     │
│       ▼                                                     │
│  ┌─────────┐    ┌──────────┐    ┌────────────────────────┐ │
│  │  Lexer  │───▶│  Parser  │───▶│  Type Checker (Schema) │ │
│  └─────────┘    └──────────┘    └────────────────────────┘ │
│                                          │                  │
│                                          ▼                  │
│                               ┌──────────────────────┐     │
│                               │   Rust Transpiler    │     │
│                               └──────────────────────┘     │
│                                          │                  │
│                    ┌─────────────────────┤                  │
│                    ▼                     ▼                  │
│           ┌──────────────┐    ┌──────────────────────┐     │
│           │  NQP (sLM)   │    │  Polars LazyFrame     │     │
│           │  State Pred. │    │  Apache Arrow         │     │
│           └──────────────┘    │  rayon (parallel)     │     │
│                               └──────────────────────┘     │
└─────────────────────────────────────────────────────────────┘
```

---

## Roadmap

| Phase | 목표 | 핵심 구성요소 | 상태 |
|-------|------|--------------|------|
| **Phase 1** — Language Core | 언어 기반 완성 | Lexer, Parser, Type System, Pipeline Operator (`\|>`), Safe-Load (`::`) | 🔨 진행 중 |
| **Phase 2** — Execution Layer | 완전한 실행 환경 | Rust Transpiler 완성, Polars 완전 연동, 증분 컴파일, CLI (`x1zz run/check/fmt`) | 📋 설계 완료 |
| **Phase 3** — Prediction Layer | AI 예측 레이어 | SDE (Synthetic Data Engine), NQP 모델 학습, State Prediction PoC | 🔨 진행 중 |
| **Phase 4** — Copilot OS | 자연어 인터페이스 | 자연어 → 파이프라인 변환, MCP 서버, x1zz-Copilot 통합, **Main 시연** | 🔭 비전 |

> *2026년 현재 Phase 1 구현이 진행 중이다.*

---

## Current Status

| Component | Status |
|-----------|--------|
| Lexer / Parser | 🔨 구현 진행 중 |
| Pipeline Operator (`\|>`) | ✅ 설계 완료 |
| Safe-Load (`::`) | ✅ 설계 완료 |
| Type System (Schema) | 🔨 구현 진행 중 |
| Rust Transpiler | 🔨 구현 진행 중 |
| Polars LazyFrame 연동 | 🔨 구현 진행 중 |
| SDE (Synthetic Data Engine) | 📋 준비 중 |
| NQP State Prediction (PoC) | 📋 준비 중 |
| MCP 서버 | 🔭 Phase 4 |

---

## Installation

```bash
# 2026년 Phase 1 완료 후 공개 예정
# 현재 소스 빌드:
git clone https://github.com/x1zz-lang/x1zz-lang.git
cd x1zz-lang
cargo build --release
```

---

## CLI

```bash
x1zz check  src/pipeline/analysis.xzz   # 타입·스키마 검사
x1zz fmt    src/pipeline/analysis.xzz   # 포맷팅
x1zz run    src/pipeline/analysis.xzz   # 실행
x1zz run    src/pipeline/analysis.xzz --predict  # NQP 사전 예측 후 실행
x1zz emit   rust src/pipeline/analysis.xzz       # Rust 코드 출력
```

---

## Contributing

x1zzLang은 오픈소스다.  
언어 설계, Rust 구현, NQP 모델 학습, 공공 데이터 파이프라인 작성 — 모든 기여를 환영합니다

이슈와 PR은 GitHub에서.

---

## License

Apache-2.0 — 자유롭게 사용하고, 자유롭게 기여하세요

---

<div align="center">



**x1zzLang — 2026**

</div>
