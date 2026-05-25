// x1zz-compiler/src/main.rs
//
// 완전한 컴파일러 파이프라인:
//   1. std::fs::read_to_string  — 소스 파일 동적 로드
//   2. Lexer::tokenize()        — 문자 스트림 → Token 배열
//   3. Parser::parse()          — Token 배열 → Program AST
//   4. Codegen::generate()      — Program AST → Polars 흐름 문자열
//
// 하드코딩된 출력 없음. 모든 출력은 소스 파일 파싱 결과.

use std::fs;
use x1zz_compiler::{Codegen, Lexer, Parser};

fn main() {
    // ── 소스 파일 경로 (커맨드라인 인자 또는 기본값) ─────────────────────────
    let args: Vec<String> = std::env::args().collect();
    let source_path = args.get(1).map(String::as_str)
        .unwrap_or("examples/poc_script.xzz");

    // ── STEP 1: 소스 파일 읽기 ───────────────────────────────────────────────
    let source = match fs::read_to_string(source_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[x1zz ERROR] 파일 읽기 실패: '{}' — {}", source_path, e);
            std::process::exit(1);
        }
    };

    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║  x1zzLang Compiler  ·  PoC Build                         ║");
    println!("║  2026 제8회 한국코드페어 대상 목표                            ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();
    println!("▶ 입력 파일 : {}", source_path);
    println!("▶ 소스 크기 : {} bytes", source.len());
    println!();

    // ── 소스 코드 출력 ───────────────────────────────────────────────────────
    println!("┌─ 소스 코드 ──────────────────────────────────────────────────");
    for (i, line) in source.lines().enumerate() {
        println!("│ {:3} │ {}", i + 1, line);
    }
    println!("└──────────────────────────────────────────────────────────────");
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
        "┌─ Lexer 출력 ({} 토큰) ─────────────────────────────────────",
        tokens.len()
    );
    for tok in &tokens {
        println!("│  [{:>4}:{:<3}]  {:?}", tok.span.line, tok.span.col, tok.kind);
    }
    println!("└──────────────────────────────────────────────────────────────");
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

    println!("┌─ AST (Program) ──────────────────────────────────────────────");
    println!("{:#?}", program);
    println!("└──────────────────────────────────────────────────────────────");
    println!();

    // ── STEP 4: Codegen — Polars 흐름 생성 ──────────────────────────────────
    let output = Codegen::generate(&program);

    println!("┌─ Polars LazyFrame 흐름 매핑 ─────────────────────────────────");
    for line in output.lines() {
        println!("│ {}", line);
    }
    println!("└──────────────────────────────────────────────────────────────");
    println!();

    println!("✅  컴파일 완료 — Program 노드 {} 개", program.stmts.len());
}
