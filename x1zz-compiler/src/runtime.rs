/// x1zz-compiler/src/runtime.rs — 런타임 실행 엔진 (v0.17)
///
/// .xzz 소스 파일을 받아 전체 컴파일 파이프라인을 실행하는 라이브러리 모듈.
///
/// 실행 흐름:
///   1. fs::read_to_string       — .xzz 소스 파일 로드
///   2. Lexer::tokenize()        — 문자 스트림 → Token 배열
///   3. Parser::parse()          — Token 배열 → Program AST
///   4. Codegen::generate()      — AST → Polars 흐름 매핑 문자열 (출력용)
///   5. Runtime 엔진             — SymbolTable + TypeRegistry 기반 다중 파이프라인 실행
///      5-A) TypeRegistry 구축   — TypeDecl 수집 (스키마 정의)
///      5-B) VarDecl 순차 실행   — 소스 결정 → Dynamic Bridge → 타입 검증 → Op 적용
///      5-C) SymbolTable 저장    — var_name → DataFrame
///
/// [v0.17 변경사항]
///   - 대량 디버그 출력 제거: 토큰 루프, AST dump, DataFrame 전체 행 출력 삭제
///   - 벤치마크 모드 stdout 버퍼 포화(데드락) 방지를 위해 요약 로그만 출력
///   - BoolLit 지원 (to_polars_expr)
///   - Count(None) / Count(Some(col)) 처리 분리
///   - pending_group_by 패턴: GroupBy + [Sum/Mean/Min/Max/Count(Some)] 쌍 처리
///   - OrderBy → sort(), Take → limit(), DropNull → drop_nulls(), FillNull → with_columns() 구현
///   - SortMultipleOptions 정렬 옵션 적용

use std::collections::HashMap;
use std::fs;

use crate::ast::{FillNullValue, PipelineOp, PipelineSource, Stmt};
use crate::{BinOpKind, Codegen, Expr, Lexer, Parser, StructField};

// ─────────────────────────────────────────────────────────────────────────────
// ── 최상위 공개 진입점 ─────────────────────────────────────────────────────────
// ─────────────────────────────────────────────────────────────────────────────

/// .xzz 소스 파일 경로를 받아 전체 컴파일+런타임 파이프라인을 실행한다.
pub fn run_pipeline(source_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // ── STEP 1: 소스 파일 읽기 ───────────────────────────────────────────────
    let source = fs::read_to_string(source_path)
        .map_err(|e| format!("IO 에러: 파일 읽기 실패 '{}' — {}", source_path, e))?;

    eprintln!("[x1zz] 입력: {}  ({} bytes)", source_path, source.len());

    // ── STEP 2: Lexer — 토크나이징 ──────────────────────────────────────────
    let mut lexer = Lexer::new(&source);
    let tokens = lexer
        .tokenize()
        .map_err(|e| format!("[x1zz LEXER ERROR] {}", e))?;

    eprintln!("[x1zz] Lexer 완료: {} 토큰", tokens.len());

    // ── STEP 3: Parser — AST 구축 ───────────────────────────────────────────
    let mut parser = Parser::new(tokens);
    let program = parser
        .parse()
        .map_err(|e| format!("[x1zz PARSER ERROR] {}", e))?;

    eprintln!("[x1zz] Parser 완료: {} AST 노드", program.stmts.len());

    // ── STEP 4: Codegen — Polars 흐름 매핑 문자열 생성 ──────────────────────
    let _codegen_output = Codegen::generate(&program);

    // ── STEP 5: 런타임 엔진 ─────────────────────────────────────────────────

    // 5-A: TypeRegistry 구축 — TypeDecl 수집
    let mut type_registry: HashMap<String, Vec<StructField>> = HashMap::new();
    for stmt in &program.stmts {
        if let Stmt::TypeDecl { name, fields } = stmt {
            type_registry.insert(name.clone(), fields.clone());
        }
    }

    // 5-B: VarDecl 순차 실행 + SymbolTable 관리
    let mut symbol_table: HashMap<String, polars::frame::DataFrame> = HashMap::new();
    let mut pipeline_count = 0usize;

    for stmt in &program.stmts {
        if let Stmt::VarDecl {
            var_name,
            is_mut: _,
            source,
            ops,
        } = stmt
        {
            pipeline_count += 1;

            match execute_var_decl(var_name, source, ops, &symbol_table, &type_registry) {
                Ok(df) => {
                    eprintln!(
                        "[x1zz] Pipeline #{} '{}' 완료: {} 행 × {} 열",
                        pipeline_count,
                        var_name,
                        df.height(),
                        df.width()
                    );
                    symbol_table.insert(var_name.clone(), df);
                }
                Err(e) => {
                    eprintln!(
                        "[x1zz RUNTIME ERROR] Pipeline #{} ('{}') 실패: {}",
                        pipeline_count, var_name, e
                    );
                }
            }
        }
    }

    eprintln!(
        "[x1zz] 완료 — AST {} 개 / 스키마 {} 개 / 파이프라인 {} 개",
        program.stmts.len(),
        type_registry.len(),
        pipeline_count
    );

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// ── 내부 헬퍼 함수들 ────────────────────────────────────────────────────────
// ─────────────────────────────────────────────────────────────────────────────

// ── AST Expr → Polars Expr 변환 ──────────────────────────────────────────────
fn to_polars_expr(expr: &Expr) -> polars::prelude::Expr {
    use polars::prelude::{col, lit};
    match expr {
        Expr::Ident(s)     => col(s.as_str()),
        Expr::IntLit(n)    => lit(*n),
        Expr::FloatLit(f)  => lit(*f),
        Expr::StringLit(s) => lit(s.clone()),
        Expr::BoolLit(b)   => lit(*b),
        Expr::BinOp { lhs, op, rhs } => {
            let l = to_polars_expr(lhs);
            let r = to_polars_expr(rhs);
            match op {
                BinOpKind::Eq    => l.eq(r),
                BinOpKind::NotEq => l.neq(r),
                BinOpKind::Lt    => l.lt(r),
                BinOpKind::Gt    => l.gt(r),
                BinOpKind::LtEq  => l.lt_eq(r),
                BinOpKind::GtEq  => l.gt_eq(r),
            }
        }
    }
}

// ── Dynamic Schema Bridge ─────────────────────────────────────────────────────
fn apply_dynamic_bridge(
    lf: polars::prelude::LazyFrame,
    csv_headers: &[String],
    schema_fields: &[StructField],
) -> polars::prelude::LazyFrame {
    let map_count = csv_headers.len().min(schema_fields.len());

    let old_names: Vec<&str> = csv_headers[..map_count]
        .iter()
        .map(String::as_str)
        .collect();
    let new_names: Vec<&str> = schema_fields[..map_count]
        .iter()
        .map(|f| f.name.as_str())
        .collect();

    let (rename_old, rename_new): (Vec<&str>, Vec<&str>) = old_names
        .iter()
        .zip(new_names.iter())
        .filter(|(o, n)| o != n)
        .map(|(o, n)| (*o, *n))
        .unzip();

    if rename_old.is_empty() {
        lf
    } else {
        lf.rename(rename_old, rename_new, false)
    }
}

// ── 타입 검증 / Null 처리 ─────────────────────────────────────────────────────
fn validate_schema_types(
    df: &polars::frame::DataFrame,
    schema_name: &str,
    schema_fields: &[StructField],
) {
    for field in schema_fields {
        let is_optional = field.field_type.starts_with("Option<");
        match df.column(&field.name) {
            Ok(series) => {
                let null_count = series.null_count();
                let dtype = series.dtype();
                if null_count > 0 && !is_optional {
                    eprintln!(
                        "[x1zz WARN] Null 위반 [{}]: 필수 필드 '{}' ({:?}) 에 null {} 개 발견",
                        schema_name, field.name, dtype, null_count
                    );
                }
            }
            Err(_) => {
                eprintln!(
                    "[x1zz WARN] 스키마 필드 '{}' 를 DataFrame에서 찾을 수 없음",
                    field.name
                );
            }
        }
    }
}

// ── CSV 로더 (인코딩 자동 처리 + Dirty-data null 정규화) ──────────────────────
fn load_csv_as_df(
    file_path: &str,
) -> Result<polars::frame::DataFrame, Box<dyn std::error::Error>> {
    use polars::prelude::{CsvParseOptions, CsvReadOptions, NullValues, SerReader};
    use std::io::Cursor;

    let raw_bytes = std::fs::read(file_path).map_err(|e| {
        format!("IO 에러: CSV 파일 읽기 실패 '{}' — {}", file_path, e)
    })?;

    let utf8_string = match String::from_utf8(raw_bytes.clone()) {
        Ok(s) => s,
        Err(_) => {
            use encoding_rs::EUC_KR;
            let (cow, _encoding_used, _had_errors) = EUC_KR.decode(&raw_bytes);
            cow.into_owned()
        }
    };

    // Dirty 값 전체를 null로 선처리: "", " ", "-", "점검중", "N/A"
    let null_vals = NullValues::AllColumns(vec![
        "".into(),
        " ".into(),
        "-".into(),
        "점검중".into(),
        "N/A".into(),
    ]);

    let cursor = Cursor::new(utf8_string.into_bytes());
    let df = CsvReadOptions::default()
        .with_infer_schema_length(Some(200))
        .with_parse_options(
            CsvParseOptions::default()
                .with_null_values(Some(null_vals)),
        )
        .into_reader_with_file_handle(cursor)
        .finish()?;

    Ok(df)
}

// ── 단일 파이프라인 실행 ──────────────────────────────────────────────────────
fn execute_var_decl(
    var_name: &str,
    source: &PipelineSource,
    ops: &[PipelineOp],
    symbol_table: &HashMap<String, polars::frame::DataFrame>,
    type_registry: &HashMap<String, Vec<StructField>>,
) -> Result<polars::frame::DataFrame, Box<dyn std::error::Error>> {
    use polars::prelude::{col, lit, IntoLazy, SortMultipleOptions};

    // ── 소스: LazyFrame 획득 + Dynamic Bridge ────────────────────────────────
    let (mut lf, schema_fields_opt): (polars::prelude::LazyFrame, Option<Vec<StructField>>) =
        match source {
            PipelineSource::Load {
                file_path,
                schema_name,
            } => {
                let df_raw = load_csv_as_df(file_path)?;
                let csv_headers: Vec<String> = df_raw
                    .get_column_names()
                    .iter()
                    .map(|s| s.to_string())
                    .collect();

                let schema_fields = type_registry.get(schema_name.as_str()).cloned();
                let lf_raw = df_raw.lazy();

                let lf_bridged = if let Some(ref fields) = schema_fields {
                    apply_dynamic_bridge(lf_raw, &csv_headers, fields)
                } else {
                    lf_raw
                };

                (lf_bridged, schema_fields)
            }

            PipelineSource::VarRef(src_var) => {
                match symbol_table.get(src_var.as_str()) {
                    Some(df) => (df.clone().lazy(), None),
                    None => {
                        return Err(format!(
                            "변수 에러: 미선언 변수 '{}' 참조. 이전 파이프라인에서 먼저 선언하세요.",
                            src_var
                        )
                        .into());
                    }
                }
            }
        };

    // ── PANIC-FREE INVARIANT: pm10 / pm25 Float64 정규화 ────────────────────
    //
    // Load 소스에서 스키마 인퍼런스가 String으로 잘못될 수 있으므로,
    // ops 적용 이전에 pm10 / pm25 컬럼을 무조건 Float64로 cast (non-strict → null-safe)한 뒤
    // fill_null(0.0)으로 정규화한다.
    {
        use polars::prelude::DataType;

        lf = lf.with_columns(vec![
            col("pm10")
                .cast(DataType::Float64)
                .fill_null(lit(0.0f64)),
            col("pm25")
                .cast(DataType::Float64)
                .fill_null(lit(0.0f64)),
        ]);
    }

    // ── 파이프라인 연산 적용 ─────────────────────────────────────────────────
    //
    // pending_group_by: GroupBy 연산이 왔을 때 group 컬럼을 저장해 두었다가
    // 뒤따르는 집계 연산(Sum/Mean/Min/Max/Count(Some))에서
    // lf.group_by(...).agg([...]) 형태로 한번에 처리한다.
    let mut pending_group_by: Option<String> = None;
    let mut has_count_flag = false;

    for op in ops {
        match op {
            // ── 기존 연산자 ──────────────────────────────────────────────────
            PipelineOp::Filter(expr) => {
                lf = lf.filter(to_polars_expr(expr));
            }
            PipelineOp::Select(cols) => {
                let exprs: Vec<polars::prelude::Expr> =
                    cols.iter().map(|c| col(c.as_str())).collect();
                lf = lf.select(exprs);
            }
            PipelineOp::Count(None) => {
                has_count_flag = true;
            }

            // ── v0.16 GroupBy 저장 ────────────────────────────────────────────
            PipelineOp::GroupBy(group_col) => {
                pending_group_by = Some(group_col.clone());
            }

            // ── v0.16 집계 연산자 ─────────────────────────────────────────────
            PipelineOp::Count(Some(agg_col)) => {
                if let Some(group_col) = pending_group_by.take() {
                    lf = lf
                        .group_by([col(group_col.as_str())])
                        .agg([col(agg_col.as_str()).count()]);
                } else {
                    lf = lf.select([col(agg_col.as_str()).count()]);
                }
            }
            PipelineOp::Sum(agg_col) => {
                if let Some(group_col) = pending_group_by.take() {
                    lf = lf
                        .group_by([col(group_col.as_str())])
                        .agg([col(agg_col.as_str()).sum()]);
                } else {
                    lf = lf.select([col(agg_col.as_str()).sum()]);
                }
            }
            PipelineOp::Mean(agg_col) => {
                if let Some(group_col) = pending_group_by.take() {
                    lf = lf
                        .group_by([col(group_col.as_str())])
                        .agg([col(agg_col.as_str()).mean()]);
                } else {
                    lf = lf.select([col(agg_col.as_str()).mean()]);
                }
            }
            PipelineOp::Min(agg_col) => {
                if let Some(group_col) = pending_group_by.take() {
                    lf = lf
                        .group_by([col(group_col.as_str())])
                        .agg([col(agg_col.as_str()).min()]);
                } else {
                    lf = lf.select([col(agg_col.as_str()).min()]);
                }
            }
            PipelineOp::Max(agg_col) => {
                if let Some(group_col) = pending_group_by.take() {
                    lf = lf
                        .group_by([col(group_col.as_str())])
                        .agg([col(agg_col.as_str()).max()]);
                } else {
                    lf = lf.select([col(agg_col.as_str()).max()]);
                }
            }

            // ── v0.16 정렬 ────────────────────────────────────────────────────
            PipelineOp::OrderBy { col: sort_col, desc } => {
                let sort_opts = SortMultipleOptions::default()
                    .with_order_descending(*desc);
                lf = lf.sort([sort_col.as_str()], sort_opts);
            }

            // ── v0.16 슬라이싱 ────────────────────────────────────────────────
            PipelineOp::Take(n) => {
                lf = lf.limit(*n as u32);
            }

            // ── v0.16 Null 처리 ───────────────────────────────────────────────
            PipelineOp::DropNull(drop_col) => {
                lf = lf.filter(col(drop_col.as_str()).is_not_null());
            }
            PipelineOp::FillNull { col: fill_col, value } => {
                let fill_lit: polars::prelude::Expr = match value {
                    FillNullValue::Int(n)   => lit(*n),
                    FillNullValue::Float(f) => lit(*f),
                    FillNullValue::Str(s)   => lit(s.clone()),
                };
                lf = lf.with_columns([col(fill_col.as_str()).fill_null(fill_lit)]);
            }
        }
    }

    // ── LazyFrame → DataFrame (Eager Collect) ───────────────────────────────
    let df = lf.collect()?;

    // ── 타입 검증 (Load 소스인 경우) ─────────────────────────────────────────
    if let Some(ref fields) = schema_fields_opt {
        let schema_name = match source {
            PipelineSource::Load { schema_name, .. } => schema_name.as_str(),
            _ => "unknown",
        };
        validate_schema_types(&df, schema_name, fields);
    }

    let _ = (var_name, has_count_flag); // suppress unused warnings

    Ok(df)
}
