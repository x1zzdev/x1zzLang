# x1zzLang

**데이터 분석 자체가 문법인 언어.**
겉은 스크립트, 속은 컴파일.

```xzz
v data = load("sales.csv") :: SalesSchema

data
  |> filter(col("price") > 100)
  |> groupBy("region")
  |> mean("price")
```

`import`도, `main()`도 없다. 파일 상단부터 실행된다.

---

## Why x1zzLang

기존 데이터 분석 환경의 문제는 언어가 아니라 도구들의 조합으로 해결하려는 구조 자체에 있다.

- 런타임에서 터지는 타입 에러
- 실행 전까지 알 수 없는 결과
- 라이브러리 의존성과 보일러플레이트

x1zzLang은 이 문제를 **언어 설계 수준에서** 해결한다.
`filter`, `groupBy`, `mean`은 라이브러리 함수가 아니라 언어 문법이다.

---

## Core Design

### Safe-Load

```xzz
v data = load("sales.csv") :: SalesSchema
```

스키마를 선언하면 컬럼 존재 여부와 타입 일치를 **컴파일 시점에 검증**한다.
데이터를 불러오는 순간, 이미 검증된 상태다.

### Pipeline Operator

```xzz
data
  |> filter(col("price") > 100)
  |> sum("price")
```

파이프라인은 문법 설탕이 아니다.
내부적으로 Polars LazyFrame 연산 그래프로 변환되어 최적화된다.

### Compile-time Type System

`.xzz` 코드는 Rust로 트랜스파일되어 네이티브 바이너리로 실행된다.

```
.xzz → Rust (transpile) → Native Binary (Polars)
```

---

## Neural Query Planner (x1zz-Copilot)

x1zzLang의 AI 레이어는 코드를 생성하는 어시스턴트가 아니다.

**코드를 실행하기 전에 데이터의 상태 변화를 예측한다.**

```xzz
data |> filter(col("price") > 100) |> sum("price")
```

```json
{
  "rows_before": 1000,
  "rows_after":  230,
  "sum(price)":  "~10,350,000 (est.)"
}
```

이 예측값은 실제 실행 결과가 아니라, 파이프라인 구조와 데이터 분포를 기반으로 한 **사전 추론**이다.
GitHub Copilot이 코드를 완성하는 도구라면, x1zz-Copilot은 **실행 결과를 이해하는 컴파일러의 일부**다.

---

## Status

| Component | Status |
|---|---|
| Lexer / Parser | 구현 진행 중 |
| Pipeline Operator (`\|>`) | 설계 완료 |
| Safe-Load (`::`) | 설계 완료 |
| Polars 연동 | 구현 진행 중 |
| State Prediction (PoC) | 준비 중 |

---

## Roadmap

| Phase | Scope |
|---|---|
| Phase 1 — Language Core | Lexer, Parser, Type System, Pipeline, Safe-Load |
| Phase 2 — Execution Layer | Rust transpile, Polars 완전 연동, 증분 컴파일 |
| Phase 3 — Prediction Layer | Synthetic Data Engine, State Prediction 모델 학습 |
| Phase 4 — Copilot OS | 자연어 → 파이프라인 변환, MCP 서버 |

---

## Installation

```bash
준비중
```



---

## License

Apache-2.0