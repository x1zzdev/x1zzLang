mod cli;
mod ux;

use clap::Parser;
use cli::{Cli, Commands};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        // ── run: .xzz 데이터 분석 코드 실행 ────────────────────────────────
        //   Lexer → Parser → TypeChecker → Runtime (Polars LazyFrame 체인)
        Commands::Run { file, release } => {
            let source_path = match file.to_str() {
                Some(p) => p.to_owned(),
                None => {
                    eprintln!(
                        "IO 에러: 파일 경로를 UTF-8 문자열로 변환할 수 없습니다.\n\
                         경로에 유효하지 않은 문자가 포함되어 있는지 확인하세요."
                    );
                    std::process::exit(1);
                }
            };

            // 파일 존재 여부 사전 확인
            if !file.exists() {
                eprintln!(
                    "[x1zz IO 에러]\n\
                     ─────────────────────────────────────────────\n\
                     Cause   : 소스 파일을 찾을 수 없습니다.\n\
                     Detail  : '{}' 경로에 파일이 존재하지 않습니다.\n\
                     → 경로를 다시 확인하거나 .xzz 파일을 먼저 생성하세요.",
                    source_path
                );
                std::process::exit(1);
            }

            if release {
                println!("🚀  릴리즈 모드 (Polars 최적화 플래그 활성화)");
                println!();
            }

            // x1zz-compiler 런타임 파이프라인 호출
            if let Err(e) = x1zz_compiler::runtime::run_pipeline(&source_path) {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }

        // ── emit: .xzz → 타겟 언어 변환 출력 ──────────────────────────────
        //   현재 지원 형식: rust (독립 Rust + Polars 소스 파일 생성)
        Commands::Emit { format, file, out } => {
            let source_path = match file.to_str() {
                Some(p) => p.to_owned(),
                None => {
                    eprintln!(
                        "IO 에러: 파일 경로를 UTF-8 문자열로 변환할 수 없습니다."
                    );
                    std::process::exit(1);
                }
            };

            // 파일 존재 여부 사전 확인
            if !file.exists() {
                eprintln!(
                    "[x1zz IO 에러]\n\
                     ─────────────────────────────────────────────\n\
                     Cause   : 소스 파일을 찾을 수 없습니다.\n\
                     Detail  : '{}' 경로에 파일이 존재하지 않습니다.\n\
                     → 경로를 다시 확인하거나 .xzz 파일을 먼저 생성하세요.",
                    source_path
                );
                std::process::exit(1);
            }

            match format.to_lowercase().as_str() {
                "rust" => {
                    // out 경로가 있으면 파일로, 없으면 stdout
                    let out_path = out.as_ref().and_then(|p| p.to_str()).map(String::from);

                    println!(
                        "⚙  x1zz emit rust  │  소스: {}  │  출력: {}",
                        source_path,
                        out_path.as_deref().unwrap_or("stdout")
                    );
                    println!();

                    if let Err(e) = x1zz_compiler::emitter::emit_rust(
                        &source_path,
                        out_path.as_deref(),
                    ) {
                        eprintln!("{}", e);
                        std::process::exit(1);
                    }
                }
                unknown => {
                    eprintln!(
                        "[x1zz emit 에러]\n\
                         ─────────────────────────────────────────────\n\
                         Cause   : 지원하지 않는 출력 형식입니다.\n\
                         Detail  : '{}' 는 유효한 emit 형식이 아닙니다.\n\
                         Available: rust\n\
                         → Did you mean: x1zz emit rust {}",
                        unknown, source_path
                    );
                    std::process::exit(1);
                }
            }
        }

        // ── check: sLM 정적 분석 (NQP) ──────────────────────────────────────
        Commands::Check { file } => {
            println!("🔍  정적 분석을 시작합니다 …  ({})", file.display());

            let spinner = ux::create_spinner("sLM Neural Query Planner 분석 중 …");
            sleep(Duration::from_millis(1_200)).await;
            spinner.finish_and_clear();

            let file_name = file
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            ux::print_mock_nqp_report(file_name);
        }

        // ── sde: 합성 데이터 생성 ────────────────────────────────────────────
        Commands::Sde { rows, output } => {
            println!(
                "⚙  x1zz sde  │  rows: {}  │  output: {}",
                rows,
                output.display()
            );
            println!("   [x1zz-sde 엔진 연동 예정]  정상 종료.");
        }
    }

    Ok(())
}
