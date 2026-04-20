# x1zzLang

> **데이터 분석을 위한 도구가 아니라, 데이터 분석 자체가 문법인 언어**

---

## 🚀 Introduction

x1zzLang은 고성능 데이터 분석을 위해 설계된 새로운 프로그래밍 언어다.

파일 상단부터 바로 실행되는 스크립트 형태지만,
내부적으로는 Rust + Polars로 컴파일되어 네이티브 수준의 성능을 낸다.

```xzz
v data = load("sales.csv") :: SalesSchema;

data
  |> filter(col("price") > 100)
  |> groupBy("region")
  |> mean("price")
  |> plot.bar(x: "region", y: "price");
```

import도, main()도 없다.
데이터 분석 자체가 곧 코드다.

---

## ⚡ Why x1zzLang

기존 데이터 분석 환경은 구조적으로 불편하다.

* pandas / numpy / matplotlib 조합
* 런타임에서 터지는 에러
* 실행해보기 전까지 결과를 모름

x1zzLang은 이 문제를 **언어 설계 수준에서** 해결하려고 한다.

---

## 🧠 Core Principles

### 1. Data-First Grammar

filter, groupBy, mean 같은 연산이
라이브러리가 아니라 **언어 문법**이다.

---

### 2. Compile-time Safety

```xzz
v data = load("sales.csv") :: SalesSchema;
```

스키마를 기반으로:

* 컬럼 존재 여부
* 타입 일치 여부

를 **컴파일 시점에 검증**한다.

---

### 3. Zero-cost Abstraction

파이프라인 문법은 단순한 문법 설탕이 아니다.

```text
.xzz → Rust → Native Binary
```

모든 코드는 최적화된 Rust 코드로 변환된다.

---

### 4. Pipeline as Default

```xzz
data |> filter(...) |> mean(...)
```

데이터 흐름이 코드 구조 그대로 드러난다.

---

## 🔍 Additional Capability: State Prediction (Experimental)

최근 추가된 기능 중 하나는
**코드를 실행하기 전에 데이터 변화를 예측하는 것**이다.

예를 들어:

```xzz
data |> filter(col("price") > 100) |> mean("price")
```

이 코드를 실행하기 전에:

* row 수가 줄어드는지
* 평균이 올라가는지
* 분포가 어떻게 바뀌는지

를 추정한다.

```json
{
  "rows": "1000 -> 230",
  "mean(price)": "12000 -> 45000"
}
```

이 값은 실제 결과가 아니라
**코드를 기반으로 한 예측**이다.

> 정확한 값보다 “변화의 방향”을 보는 것이 목적이다.

---

## 🧩 How It Works (Simplified)

```text
.xzz code
   ↓
Compiler (Rust)
   ↓
Execution (Polars)
```

State Prediction은 이 흐름 옆에서 동작하는 **보조 레이어**다.

```text
code → (AI) → predicted state
     → (runtime) → actual result
```

---

## 🛠️ Key Features

* DataFrame 연산이 언어 문법으로 내장
* 스키마 기반 Safe-Load
* 컴파일 타임 타입 검사
* Rust + Polars 기반 실행
* 파이프라인 중심 코드 구조
* (실험적) 실행 전 결과 예측

---

## 📦 Installation (Planned)

```bash
git clone https://github.com/x1zz/x1zzLang
cd x1zzLang
cargo build
```

---

## ▶️ Usage

```bash
x1zz run analysis.xzz
```

---

## 🧱 Current Status

초기 구현 단계.

* 언어: 설계 + 일부 구현 진행 중
* 컴파일러: 기본 구조 설계 완료
* AI: State Prediction PoC 준비 중

---

## 🧭 Roadmap

### Phase 1 — Language Core

* Parser / Type System / Pipeline

### Phase 2 — Execution Layer

* Rust + Polars 통합
* Safe-Load 완성

### Phase 3 — Prediction Layer

* Synthetic Data Engine
* State Prediction 모델

### Phase 4 — Copilot

* 자연어 → 파이프라인 변환

---

## 🤝 Contributors

* Seowoo Jang

---

## 📜 License

TBD
