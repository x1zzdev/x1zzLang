# x1zzLang Runtime Completion Sprint — 최종 보고서

> 작성일: 2026-06-01  
> 버전: v0.16  
> 목표: README 예시 파이프라인 전체 실행 지원

---

## 1. 아키텍처 감사 결과

| 연산 | AST | Parser | Runtime | Emitter | 스프린트 전 상태 |
|------|-----|--------|---------|---------|----------------|
| `load()` | ✅ | ✅ | ✅ | ✅ | 완료 |
| `select()` | ✅ | ✅ | ✅ | ✅ | 완료 |
| `filter()` | ✅ | ✅ | ✅ | ✅ | 완료 |
| `count()` | ✅ (단독만) | ✅ (단독만) | ✅ (단독만) | ✅ (단독만) | 부분 완료 |
| `groupBy()` | ❌ | ❌ | ❌ | ❌ | **미구현** |
| `sum()` | ❌ | ❌ | ❌ | ❌ | **미구현** |
| `mean()` | ❌ | ❌ | ❌ | ❌ | **미구현** |
| `min()` | ❌ | ❌ | ❌ | ❌ | **미구현** |
| `max()` | ❌ | ❌ | ❌ | ❌ | **미구현** |
| `orderBy()` | ❌ | ❌ | ❌ | ❌ | **미구현** |
| `take()` | ❌ | ❌ | ❌ | ❌ | **미구현** |
| `dropNull()` | ❌ | ❌ | ❌ | ❌ | **미구현** |
| `fillNull()` | ❌ | ❌ | ❌ | ❌ | **미구현** |
| `col("x")` 구문 | ❌ | ❌ | ❌ | ❌ | **미구현** |
| `true/false` 리터럴 | ❌ | ❌ | ❌ | ❌ | **미구현** |
| 숫자 언더스코어 (`1_200_000`) | ❌ | — | — | — | **미구현** |

---

## 2. 수정된 파일

| 파일 | 변경 유형 |
|------|----------|
| `x1zz-compiler/src/token.rs` | 전면 재작성 — 12개 새 토큰 추가 |
| `x1zz-compiler/src/lexer.rs` | 전면 재작성 — 키워드 매핑, 언더스코어, 불리언 추가 |
| `x1zz-compiler/src/ast.rs` | 전면 재작성 — 새 열거형 및 변형 추가 |
| `x1zz-compiler/src/lib.rs` | `FillNullValue` public re-export 추가 |
| `x1zz-compiler/src/parser.rs` | 전면 재작성 — 새 파싱 규칙, col() 지원, 테스트 추가 |
| `x1zz-compiler/src/codegen.rs` | 전면 재작성 — BoolLit, 9개 새 Op 코드 생성 |
| `x1zz-compiler/src/runtime.rs` | 전면 재작성 — pending_group_by 패턴, 모든 새 Op 실행 |
| `x1zz-compiler/src/emitter.rs` | 전면 재작성 — 새 Op Rust 코드 생성 |

---

## 3. 추가된 AST 노드

### `Expr` 변형 (1개)
```rust
BoolLit(bool)   // true / false 리터럴
```

### `FillNullValue` 열거형 (신규)
```rust
pub enum FillNullValue {
    Int(i64),
    Float(f64),
    Str(String),
}
```

### `PipelineOp` 변형 (9개 신규 + 1개 변경)
```rust
// 변경
Count(Option<String>)  // None: 전체 행 수, Some(col): 그룹 집계

// 신규 9종
GroupBy(String),
Sum(String),
Mean(String),
Min(String),
Max(String),
OrderBy { col: String, desc: bool },
Take(i64),
DropNull(String),
FillNull { col: String, value: FillNullValue },
```

---

## 4. 추가된 파서 규칙

| 규칙 | 설명 |
|------|------|
| `col("x")` → `Expr::Ident("x")` | Ident "col" + LParen 패턴 감지 |
| `true` / `false` → `Expr::BoolLit` | TokenKind::True/False 처리 |
| `1_200_000` → `IntLit(1200000)` | Lexer read_number에서 `_` 무시 |
| `groupBy("col")` → `PipelineOp::GroupBy` | 문자열 인자 1개 |
| `sum("col")` → `PipelineOp::Sum` | 문자열 인자 1개 |
| `mean("col")` → `PipelineOp::Mean` | 문자열 인자 1개 |
| `min("col")` → `PipelineOp::Min` | 문자열 인자 1개 |
| `max("col")` → `PipelineOp::Max` | 문자열 인자 1개 |
| `orderBy("col", desc: true)` → `PipelineOp::OrderBy` | 문자열 + `desc:` 불리언 |
| `take(n)` → `PipelineOp::Take` | 정수 인자 1개 |
| `dropNull("col")` → `PipelineOp::DropNull` | 문자열 인자 1개 |
| `fillNull("col", value)` → `PipelineOp::FillNull` | 문자열 + Int/Float/Str 값 |
| `count("col")` → `PipelineOp::Count(Some(...))` | 선택적 인자로 변경 |

---

## 5. 런타임 구현된 연산

### pending_group_by 패턴
GroupBy 연산은 컬럼명을 `pending_group_by: Option<String>`에 저장하고,
뒤따르는 집계 연산(Sum/Mean/Min/Max/Count(Some))에서 소비하여
`lf.group_by([...]).agg([...])` 단일 체인으로 실행.

```
PipelineOp::GroupBy(col)        → pending_group_by = Some(col)
PipelineOp::Sum(agg_col)        → group_by(pending).agg([col.sum()]) 또는 select([col.sum()])
PipelineOp::Mean(agg_col)       → group_by(pending).agg([col.mean()]) 또는 select([col.mean()])
PipelineOp::Min(agg_col)        → group_by(pending).agg([col.min()]) 또는 select([col.min()])
PipelineOp::Max(agg_col)        → group_by(pending).agg([col.max()]) 또는 select([col.max()])
PipelineOp::Count(Some(col))    → group_by(pending).agg([col.count()]) 또는 select([col.count()])
PipelineOp::OrderBy{col, desc}  → lf.sort([col], SortMultipleOptions::default().with_order_descending(desc))
PipelineOp::Take(n)             → lf.limit(n as u32)
PipelineOp::DropNull(col)       → lf.filter(col.is_not_null())   // Polars 0.53 Selector API 우회
PipelineOp::FillNull{col, val}  → lf.with_columns([col.fill_null(lit(val))])
```

---

## 6. Emitter 변경사항

- `to_typed_polars_expr`: `BoolLit(b)` → `lit(true)` / `lit(false)` 생성
- `validate_op_columns`: 9개 새 Op에 대해 컬럼 인자 유효성 검사 추가
- 코드 생성 루프: `pending_group_col: Option<String>` 사용하여 GroupBy+집계를 단일 `.group_by([...]).agg([...])` 코드 블록으로 생성

---

## 7. 추가된 테스트

### Lexer 테스트 (4개)
| 테스트명 | 내용 |
|---------|------|
| `test_new_pipeline_keywords` | groupBy, sum, mean, min, max, orderBy, take, dropNull, fillNull 토큰 확인 |
| `test_boolean_keywords` | `true`, `false` → True/False 토큰 |
| `test_number_underscore` | `1_200_000` → IntLiteral(1200000) |
| `test_desc_keyword` | `desc` → Desc 토큰 |

### Parser 테스트 (7개)
| 테스트명 | 내용 |
|---------|------|
| `test_col_function_in_filter` | `filter(col("income") < 1200000)` → `Filter(BinOp{Ident("income"), Lt, IntLit})` |
| `test_boolean_literal_in_filter` | `filter(col("support") == false)` → `Filter(BinOp{..., BoolLit(false)})` |
| `test_group_by_and_count` | `groupBy("region") \|> count("population")` → `[GroupBy, Count(Some)]` |
| `test_mean_orderby_take` | `mean + orderBy(desc:true) + take(10)` 파싱 |
| `test_drop_null_and_fill_null` | `dropNull + fillNull(int)` 파싱 |
| `test_sum_min_max_parse` | `sum + min + max` 파싱 |
| `test_readme_full_pipeline` | README 전체 7-연산 파이프라인 파싱 검증 |

**전체 테스트: 25/25 통과**
```
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

---

## 8. 잔존 런타임 갭

| 항목 | 상태 | 비고 |
|------|------|------|
| `plot()` | 미구현 (범위 외) | 스프린트 명시적 제외 |
| 복합 정렬 (`orderBy` 다중 컬럼) | 미구현 | 단일 컬럼만 지원 |
| `join()` | 미구현 | AST에 없음 |
| `withColumn()` / 파생 컬럼 | 미구현 | AST에 없음 |
| 집계 후 필터 (`having`에 해당) | 미구현 | `groupBy + filter` 체인 미지원 |
| `fillNull` 다중 컬럼 | 미구현 | 단일 컬럼만 지원 |
| Window 함수 | 미구현 | AST에 없음 |

---

## 9. 권장 다음 스프린트

### Sprint 2: 복합 파이프라인 & 타입 추론
1. **다중 orderBy 컬럼**: `orderBy(["col1", "col2"], desc: [true, false])`
2. **집계 후 필터 (having)**: `groupBy + having(expr)` 체인
3. **withColumn**: 파생 컬럼 생성 `|> withColumn("new", col("a") + col("b"))`
4. **join**: `|> join(other_var, on: "key")`
5. **런타임 타입 추론 강화**: 컬럼 dtype → x1zz 타입 자동 매핑

### Sprint 3: CLI & 사용성
1. `x1zz run <file>` CLI 명령 완성
2. 에러 메시지 개선 (행/열 위치 포함)
3. `--dry-run` 플래그 (AST만 출력, 실행 없이)
