// x1zz-compiler/src/main.rs  (v0.15)
//
// 컴파일러 직접 실행 엔트리포인트.
// 전체 컴파일+런타임 파이프라인은 runtime::run_pipeline() 으로 위임한다.
//
// 직접 실행 예:
//   cargo run -p x1zz-compiler -- examples/poc_script.xzz

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let source_path = args
        .get(1)
        .map(String::as_str)
        .unwrap_or("examples/poc_script.xzz");

    if let Err(e) = x1zz_compiler::runtime::run_pipeline(source_path) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
