// x1zz-compiler/src/main.rs
//
// 완전한 실행 파이프라인:
//   1. fs::read_to_string       — .xzz 소스 파일 로드
//   2. Lexer::tokenize()        — 문자 스트림 → Token 배열
//   3. Parser::parse()          — Token 배열 → Program AST
//   4. Codegen::generate()      — AST → Polars 흐름 매핑 문자열 (출력용)
//   5. Polars 런타임 엔진        — AST 기반 실제 LazyFrame CSV 실행 → DataFrame 출력
//
// 글로브 임포트(use polars::prelude::*) 금지
//   → x1zz_compiler::Expr와 polars::prelude::Expr 이름 충돌 방지

use std::fs;
use x1zz_compiler::{Codegen, Lexer, Parser};

// ── 한글 컬럼 → 영어 AST 식별자 브릿지 ──────────────────────────────────────
// seoul_air_2026.csv 실제 헤더 기준 (읽기 전용 상수 — 런타임 참조)
const BRIDGE_OLD: &[&str] = &[
    "일시",
    "구분",
    "미세먼지(PM10)",
    "초미세먼지(PM25)",
];
const BRIDGE_NEW: &[&str] = &["date", "station", "pm10", "pm25"];

// ── AST Expr → polars Expr 변환 (재귀) ──────────────────────────────────────
// polars는 구체 임포트만 사용하여 Expr 이름 충돌 회피
fn to_polars_expr(expr: &x1zz_compiler::Expr) -> polars::prelude::Expr {
    use polars::prelude::{col, lit};
    match expr {
        x1zz_compiler::Expr::Ident(s) => col(s.as_str()),
        x1zz_compiler::Expr::IntLit(n) => lit(*n),
        x1zz_compiler::Expr::FloatLit(f) => lit(*f),
        x1zz_compiler::Expr::StringLit(s) => lit(s.clone()),
        x1zz_compiler::Expr::BinOp { lhs, op, rhs } => {
            let l = to_polars_expr(lhs);
            let r = to_polars_expr(rhs);
            match op {
                x1zz_compiler::BinOpKind::Eq    => l.eq(r),
                x1zz_compiler::BinOpKind::NotEq => l.neq(r),
                x1zz_compiler::BinOpKind::Lt    => l.lt(r),
                x1zz_compiler::BinOpKind::Gt    => l.gt(r),
                x1zz_compiler::BinOpKind::LtEq  => l.lt_eq(r),
                x1zz_compiler::BinOpKind::GtEq  => l.gt_eq(r),
            }
        }
    }
}

// ── Polars 런타임 엔진 ────────────────────────────────────────────────────────
// 인코딩 처리 전략:
//   1. 파일을 raw bytes로 읽음
//   2. UTF-8 직접 파싱 시도 → 실패 시 EUC-KR(CP949)으로 디코딩
//   3. UTF-8 바이트를 Cursor<Vec<u8>>로 감싸서 CsvReader에 전달
fn execute_pipeline(
    file_path: &str,
    schema_name: &str,
    ops: &[x1zz_compiler::PipelineOp],
) -> Result<polars::frame::DataFrame, Box<dyn std::error::Error>> {
    use polars::prelude::{col, CsvParseOptions, CsvReadOptions, IntoLazy, NullValues, SerReader};
    use std::io::Cursor;

    println!(
        "  ▶ 런타임: '{}' → 스키마 '{}'",
        file_path, schema_name
    );

    // STEP 5-A: raw bytes 읽기
    let raw_bytes = std::fs::read(file_path)?;

    // STEP 5-B: UTF-8 시도 → 실패 시 EUC-KR(CP949) 디코딩
    let utf8_string = match String::from_utf8(raw_bytes.clone()) {
        Ok(s) => {
            println!("  ▶ 인코딩: UTF-8 직접 사용");
            s
        }
        Err(_) => {
            use encoding_rs::EUC_KR;
            let (cow, encoding_used, had_errors) = EUC_KR.decode(&raw_bytes);
            println!(
                "  ▶ 인코딩: {} → UTF-8 변환 (손실 여부: {})",
                encoding_used.name(),
                had_errors
            );
            cow.into_owned()
        }
    };

    // STEP 5-C: UTF-8 바이트를 Cursor로 감싸서 CsvReadOptions로 전달
    // polars 0.53 API: CsvReadOptions + CsvParseOptions + into_reader_with_file_handle
    let cursor = Cursor::new(utf8_string.into_bytes());
    let df_raw = CsvReadOptions::default()
        .with_infer_schema_length(Some(200))
        .with_parse_options(
            CsvParseOptions::default()
                .with_null_values(Some(NullValues::AllColumnsSingle("-".into()))),
        )
        .into_reader_with_file_handle(cursor)
        .finish()?;

    // STEP 5-D: DataFrame → LazyFrame + 한글 컬럼 → 영어 AST 식별자 이름 변경
    let mut lf = df_raw.lazy();
    lf = lf.rename(BRIDGE_OLD.iter().copied(), BRIDGE_NEW.iter().copied(), false);

    // STEP 5-E: AST 파이프라인 연산 순차 적용
    let mut has_count = false;
    for op in ops {
        match op {
            x1zz_compiler::PipelineOp::Filter(expr) => {
                println!("  ▶ filter({}) 적용", x1zz_compiler::Codegen::expr_to_xzz(expr));
                lf = lf.filter(to_polars_expr(expr));
            }
            x1zz_compiler::PipelineOp::Select(cols) => {
                println!("  ▶ select([{}]) 적용", cols.join(", "));
                let exprs: Vec<polars::prelude::Expr> =
                    cols.iter().map(|c| col(c.as_str())).collect();
                lf = lf.select(exprs);
            }
            x1zz_compiler::PipelineOp::Count => {
                println!("  ▶ count 적용 (collect 후 행 수 출력)");
                has_count = true;
            }
        }
    }

    // STEP 5-F: LazyFrame 실행 (Eager Collect)
    let df = lf.collect()?;

    if has_count {
        println!("\n  ✅ count 결과: {} 행", df.height());
    }

    Ok(df)
}

// ─────────────────────────────────────────────────────────────────────────────

fn main() {
    // ── 소스 파일 경로 (커맨드라인 인자 또는 기본값) ─────────────────────────
    let args: Vec<String> = std::env::args().collect();
    let source_path = args
        .get(1)
        .map(String::as_str)
        .unwrap_or("examples/poc_script.xzz");

    // ── STEP 1: 소스 파일 읽기 ───────────────────────────────────────────────
    let source = match fs::read_to_string(source_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[x1zz ERROR] 파일 읽기 실패: '{}' — {}", source_path, e);
            std::process::exit(1);
        }
    };

    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║  x1zzLang Compiler + Runtime  ·  PoC Build                   ║");
    println!("║  2026 제8회 한국코드페어 대상 목표                               ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();
    println!("▶ 입력 파일 : {}", source_path);
    println!("▶ 소스 크기 : {} bytes", source.len());
    println!();

    // 소스 코드 출력
    println!("┌─ 소스 코드 ────────────────────────────────────────────────────");
    for (i, line) in source.lines().enumerate() {
        println!("│ {:3} │ {}", i + 1, line);
    }
    println!("└────────────────────────────────────────────────────────────────");
    println!();

    // ── STEP 2: Lexer — 토크나이징 ──────────────────────────────────────────
    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("[x1zz LEXER ERROR] {}", e);
            std::process::exit(1);
        }
    };

    println!(
        "┌─ Lexer 출력 ({} 토큰) ──────────────────────────────────────",
        tokens.len()
    );
    for tok in &tokens {
        println!(
            "│  [{:>4}:{:<3}]  {:?}",
            tok.span.line, tok.span.col, tok.kind
        );
    }
    println!("└────────────────────────────────────────────────────────────────");
    println!();

    // ── STEP 3: Parser — AST 구축 ───────────────────────────────────────────
    let mut parser = Parser::new(tokens);
    let program = match parser.parse() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[x1zz PARSER ERROR] {}", e);
            std::process::exit(1);
        }
    };

    println!("┌─ AST (Program) ────────────────────────────────────────────────");
    println!("{:#?}", program);
    println!("└────────────────────────────────────────────────────────────────");
    println!();

    // ── STEP 4: Codegen — Polars 흐름 매핑 문자열 생성 ──────────────────────
    let codegen_output = Codegen::generate(&program);

    println!("┌─ Polars LazyFrame 흐름 매핑 ───────────────────────────────────");
    for line in codegen_output.lines() {
        println!("│ {}", line);
    }
    println!("└────────────────────────────────────────────────────────────────");
    println!();

    // ── STEP 5: 런타임 엔진 — AST 기반 실제 Polars 실행 ────────────────────
    println!("┌─ Polars 런타임 실행 ───────────────────────────────────────────");

    let mut pipeline_count = 0usize;

    for stmt in &program.stmts {
        if let x1zz_compiler::Stmt::PipelineStream {
            file_path,
            schema_name,
            ops,
        } = stmt
        {
            pipeline_count += 1;
            println!("│");
            println!("│  [Pipeline #{}]", pipeline_count);

            match execute_pipeline(file_path, schema_name, ops) {
                Ok(df) => {
                    println!("│");
                    println!(
                        "│  ── 결과 DataFrame ({} 행 × {} 열) ──────────────────────",
                        df.height(),
                        df.width()
                    );
                    let df_str = format!("{}", df);
                    for line in df_str.lines() {
                        println!("│  {}", line);
                    }
                }
                Err(e) => {
                    eprintln!(
                        "│  [x1zz RUNTIME ERROR] Pipeline #{} 실패: {}",
                        pipeline_count, e
                    );
                }
            }
            println!("│");
        }
    }

    if pipeline_count == 0 {
        println!("│  (실행 가능한 PipelineStream 없음)");
    }

    println!("└────────────────────────────────────────────────────────────────");
    println!();
    println!(
        "✅  완료 — AST 노드 {} 개 / 실행된 파이프라인 {} 개",
        program.stmts.len(),
        pipeline_count
    );
}
