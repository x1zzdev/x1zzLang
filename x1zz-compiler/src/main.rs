// x1zz-compiler/src/main.rs  (v0.18 → compile-only)
//
// 컴파일러 직접 실행 엔트리포인트 — 파싱 + AST 출력 전용.
//
// ⚠️  런타임 실행(Polars pipeline)은 x1zz-runner 바이너리를 사용하세요.
//     이 바이너리는 컴파일 단계(Lexer → Parser → Codegen)만 수행합니다.
//
// 사용 예:
//   cargo run -p x1zz-compiler -- examples/poc_script.xzz
//   cargo run -p x1zz-compiler -- examples/poc_script.xzz --verbose

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let input_path = args
        .get(1)
        .map(String::as_str)
        .unwrap_or("examples/poc_script.xzz");

    let verbose = args.iter().any(|a| a == "--verbose" || a == "-v");

    // ── 소스 파일 읽기 ──────────────────────────────────────────────────────
    let source = match std::fs::read_to_string(input_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[x1zz-compiler] IO 에러: '{}' — {}", input_path, e);
            std::process::exit(1);
        }
    };

    eprintln!(
        "[x1zz-compiler] 입력: {}  ({} bytes)",
        input_path,
        source.len()
    );

    // ── Lexer ────────────────────────────────────────────────────────────────
    let mut lexer = x1zz_compiler::Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("[x1zz-compiler LEXER ERROR] {}", e);
            std::process::exit(1);
        }
    };
    eprintln!("[x1zz-compiler] Lexer 완료: {} 토큰", tokens.len());

    if verbose {
        println!("\n⚡ STEP 1. Tokenized Stream");
        println!("{}", "─".repeat(60));
        for token in &tokens {
            println!(
                "  [{:>4}:{:<3}] {:?}",
                token.span.line, token.span.col, token.kind
            );
        }
    }

    // ── Parser ───────────────────────────────────────────────────────────────
    let mut parser = x1zz_compiler::Parser::new(tokens);
    let program = match parser.parse() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[x1zz-compiler PARSER ERROR] {}", e);
            std::process::exit(1);
        }
    };
    eprintln!(
        "[x1zz-compiler] Parser 완료: {} AST 노드",
        program.stmts.len()
    );

    if verbose {
        println!("\n⚡ STEP 2. Abstract Syntax Tree");
        println!("{}", "─".repeat(60));
        for (i, stmt) in program.stmts.iter().enumerate() {
            println!("  [{}] {:#?}", i, stmt);
        }
    }

    // ── Codegen ──────────────────────────────────────────────────────────────
    let codegen_output = x1zz_compiler::Codegen::generate(&program);
    println!("\n⚡ STEP 3. Codegen Output");
    println!("{}", "─".repeat(60));
    println!("{}", codegen_output);

    eprintln!("[x1zz-compiler] 컴파일 완료");
    eprintln!(
        "[x1zz-compiler] ℹ️  실행(run)은 'x1zz run {}' 를 사용하세요.",
        input_path
    );
}
