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

**비전문가도 코드 없이 데이터 분석을 수행할 수 있는 DSL 플랫폼.**

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Language: .xzz](https://img.shields.io/badge/Language-.xzz-orange.svg)]()
[![Backend: Polars](https://img.shields.io/badge/Backend-Polars-red.svg)]()
[![Status: v0.2.0](https://img.shields.io/badge/Status-v0.2.0-green.svg)]()

[English README](README.md)

</div>

---

## Visual IDE

[![x1zzLang Visual IDE](screenshot_visual_ide.png)](https://github.com/x1zzdev/x1zzLang-visual-ide)

`.xzz` 파이프라인을 위한 그래픽 편집 및 실행 환경.  
→ [x1zzLang Visual IDE 저장소](https://github.com/x1zzdev/x1zzLang-visual-ide)

---

## 🔥 3개의 명령어로 시작하기

```bash
x1zz new my-project       # 프로젝트 + 샘플 CSV 즉시 생성
cd my-project
x1zz import data.csv      # 스키마 자동 추론 — 타입 정의 자동 생성
x1zz run analysis.xzz     # 파이프라인 실행 + 차트 렌더링
```

Python 없이. pip install 없이. virtualenv 없이. 타입 직접 타이핑 없이.

---

## Why x1zzLang?

데이터는 어디에나 존재한다. 공공 데이터셋이 매년 공개된다.

**Python/pandas가 데이터 분석의 Microsoft라면, x1zzLang은 Apple이다.**

장벽은 데이터 가용성이 아니라 분석 접근성이다.

데이터를 단 한 행이라도 다루기 전에, 분석가는 라이브러리를 설치하고, Python 환경을 구성하고, 여러 API를 외워야 한다. 대부분의 사람들은 그 단계에서 멈춘다 — 문제가 해결 불가능해서가 아니라, 도구가 그들을 위해 만들어지지 않았기 때문이다.

| 장벽 | 문제 |
|------|------|
| 라이브러리 전제 조건 | Python / Pandas / SQL — 분석 전 코드 중심 셋업 필요 |
| 런타임 타입 에러 | 타입 불일치와 컬럼 오류가 실행 중에 발생 |
| 환경 의존성 | 셋업 마찰이 첫 결과 전에 사용자 이탈을 유발 |

x1zzLang은 코드 중심 분석을 DSL 기반 인터랙션으로 대체한다.

---

## Python vs. x1zzLang

**시나리오:** CSV 데이터셋을 필터링하고 집계합니다.

### Python (Pandas)

```python
import pandas as pd

df = pd.read_csv("data.csv")
df = df[df["pm10"] > 50]
result = df.groupby("station")["pm10"].mean()
print(result)
```

*라이브러리 설치 필요. 타입 에러는 런타임에 발생. NaN 처리는 수동.*

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

*import 없음. 스키마를 먼저 선언. `Option<T>`으로 null 안전 처리.*

| | Python (Pandas) | x1zzLang |
|--|-----------------|----------|
| 라이브러리 의존성 | `pandas`, `numpy` | 없음 (내장) |
| 타입 검증 | 런타임 | 스키마 선언 |
| Null 처리 | 수동 | `Option<T>` |

**워크플로 차이:**  
Python은 분석 한 줄을 작성하기 전에 의존성을 설치하고, CSV를 직접 열어 컬럼 타입을 추론하고, NaN을 명시적으로 처리해야 한다.  
x1zzLang은 `x1zz import`에서 시작한다 — 스키마가 자동 추론되고, null 안전성은 타입에 선언되며, 파이프라인은 즉시 실행 가능하다.

---

## ⚡ 온보딩 워크플로 (Onboarding Workflow)

**Before — Python + pandas**

```
Python 설치 → pip install pandas numpy → virtualenv 생성
→ CSV를 열어 컬럼명 수동 확인
→ dtype 매핑 직접 작성
→ NaN 값 명시적 처리
→ 분석 코드 작성 → 실행 → 런타임 에러 디버깅
→ pip install matplotlib → 플로팅 코드 작성 → 재실행
```

**After — x1zzLang**

```bash
x1zz new my-project    # 프로젝트 + 샘플 CSV 즉시 생성
cd my-project
x1zz import data.csv   # 스키마 자동 추론, 타입 블록이 main.xzz에 자동 기록
x1zz run analysis.xzz  # 파이프라인 실행 + 차트 렌더링
```

| 단계 | Before (pandas) | After (x1zzLang) |
|------|----------------|-----------------|
| 환경 설정 | pip, virtualenv, import 문 | 없음 |
| 스키마 선언 | 컬럼 수동 확인 | `x1zz import` 자동 생성 |
| Null 처리 | 명시적 NaN 처리 | 타입 정의의 `Option<T>` |
| 시각화 | matplotlib 별도 설치 | 파이프라인 내 `chart {}` 블록 |

---

## 🧠 핵심 UX 기능 (Core UX Features)

| 기능 | 설명 |
|------|------|
| **제로 셋업 실행** (`x1zz run`) | 단일 바이너리, Python 또는 라이브러리 설치 불필요 |
| **자동 스키마 추론** (`x1zz import`) | CSV 헤더와 샘플 데이터를 읽어 타입 정의와 `load` 문을 자동 생성. EUC-KR(CP949) 한글 CSV도 지원 |
| **선언형 파이프라인 DSL** (`\|>`) | `filter`, `groupBy`, `join`, `sort`, `withColumn`을 선언적 파이프 체인으로 조합 |
| **Null 안전 타입 시스템** (`Option<T>`) | 누락 데이터를 `Option<float>`으로 선언, `fillNull`로 안전하게 처리 |
| **내장 시각화** (`chart {}`) | 파이프라인 결과를 bar, line, pie, scatter 차트로 렌더링 — 외부 라이브러리 불필요 |
| **원커맨드 프로젝트 생성** (`x1zz new`) | 샘플 CSV + 실행 가능한 `example.xzz` + `x1zz.toml`을 한 번에 생성 |

---

## 기능 (Features)

| 기능 | 설명 |
|------|------|
| CSV 로딩 | 파일 수집 |
| 필터링 | 조건부 필터링 |
| 집계 | 그룹별 통계 |
| 시각화 | 결과 렌더링 |
| 컴파일러 파이프라인 | DSL → IR 변환 |
| Visual IDE | GUI 편집기 |
| 런타임 실행 | Polars 엔진 |

---

## 예제 (Example)

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

## 설치 (Installation)

### 1. 릴리스 다운로드

최신 릴리스를 다운로드:

**[https://github.com/x1zzdev/x1zzLang/releases](https://github.com/x1zzdev/x1zzLang/releases)**

### 2. 압축 해제

릴리스 패키지를 로컬 폴더에 압축 해제하세요.

### 3. 실행

```bash
x1zz run <file>
```

### 4. 설치 확인

```bash
x1zz --version
```

### 중요 사항

- Rust 또는 Cargo 불필요
- 독립 실행형 바이너리
- 의존성 내장 포함

---

## 빠른 시작 (Quick Start)

```bash
# 1. 릴리스 패키지를 다운로드하고 압축 해제
# 2. 파이프라인 실행
x1zz run <file>
# 3. 터미널에서 출력 확인
```

---

## 아키텍처 (Architecture)

> ⚠️ 개념적 개요만을 나타냅니다.

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

## 벤치마크 (Benchmark)

![x1zzLang Benchmark](benches/x1zzLang_benchmark2.png)

> *벤치마크: x1zzLang 파이프라인 실행 vs. 동일한 Pandas 파이프라인.*

---

## 현재 상태 (Current Status)

**활성 개발 중 (Active Development)**

---

## 로드맵 (Roadmap)

| Phase | 목표 |
|-------|------|
| Phase 1 — Core Language | DSL 문법, 타입 시스템, 컴파일러 파이프라인 |
| Phase 2 — Execution Layer | Polars 완전 연동, CLI 도구 |
| Phase 3 — IDE Integration | Visual IDE, 그래픽 파이프라인 편집기 |
| Phase 4 — AI Expansion | 자연어 인터페이스, AI 기반 분석 |

---

## 기여 (Contributing)

`x1zzLang`은 오픈소스 프로젝트입니다. 피드백과 제안은 언제든 환영합니다.

다만 2026년 제8회 한국코드페어 평가 기간 동안에는 저작자 동일성 및 프로젝트 무결성을 보장하기 위해 코드 기여(Pull Request)는 2026년 10월 대회 종료 시점까지 임시로 제한됩니다.

- 이슈 (버그 제보, 아이디어, 논의): 항상 환영합니다
- Pull Request (코드 기여): 대회 종료 전까지는 닫혀 있으며, 이후 재개됩니다

x1zzLang에 관심과 응원을 보내주셔서 감사합니다.

---

## 라이선스 (License)

Apache-2.0

---

<div align="center">

**x1zzLang — 2026**

</div>
