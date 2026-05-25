// x1zz-compiler/src/main.rs
// PoC 컴파일러 파이프라인 런타임:
//   1. examples/poc_script.xzz 읽기
//   2. Lexer  → Token 스트림
//   3. Parser → Program AST
//   4. Codegen → Polars LazyFrame 흐름 매핑 문자열 출력

use std::fs;
use x1zz_compiler::{Codegen, Lexer, Parser};

fn main() {
    let source_path = "examples/poc_script.xzz";

    // ── 1. 소스 파일 로드 ────────────────────────────────────────────────────
    let source = fs::read_to_string(source_path).unwrap_or_else(|e| {
        eprintln!("[x1zz] 소스 파일 로드 실패: '{}' — {}", source_path, e);
        std::process::exit(1);
    });

    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║  x1zzLang Compiler — PoC Pipeline                       ║");
    println!("║  2026 제8회 한국코드페어 대상 목표                          ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();
    println!("▶ 소스 파일 : {}", source_path);
    println!();
    println!("── 소스 코드 ──────────────────────────────────────────────");
    println!("{}", source.trim());
    println!();

    // ── 2. Lexer ─────────────────────────────────────────────────────────────
    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("[x1zz Lexer Error]  {}", e);
            std::process::exit(1);
        }
    };

    println!(
        "── Lexer 출력 (총 {} 토큰) ────────────────────────────────",
        tokens.len()
    );
    for tok in &tokens {
        println!(
            "  [{:>4}:{:<3}]  {:?}",
            tok.span.line, tok.span.col, tok.kind
        );
    }
    println!();

    // ── 3. Parser ────────────────────────────────────────────────────────────
    let mut parser = Parser::new(tokens);
    let program = match parser.parse() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[x1zz Parser Error] {}", e);
            std::process::exit(1);
        }
    };

    println!("── AST (Program) — {:#?} ──────────────────────────────────", "");
    println!("{:#?}", program);
    println!();

    // ── 4. Codegen ───────────────────────────────────────────────────────────
    let codegen = Codegen::new(program);
    let output = match codegen.generate() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[x1zz Codegen Error] {}", e);
            std::process::exit(1);
        }
    };

    println!("── Polars LazyFrame 흐름 매핑 ────────────────────────────");
    println!("{}", output);

    println!("✅  x1zzLang PoC 컴파일 완료");
}
