mod cli;
mod predict;
mod project;
mod schema;
mod ux;

use clap::Parser;
use cli::{Cli, Commands};
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        // ── run: .xzz 데이터 분석 코드 실행 (또는 NQP 예측) ────────────────
        //   기본: Lexer → Parser → TypeChecker → Runtime (Polars LazyFrame 체인)
        //   --predict: 코드 실행 없이 NQP 모델로 시맨틱 결과 예측
        Commands::Run {
            file,
            release,
            verbose,
            predict,
            output,
        } => {
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

            let output_csv: Option<String> = output
                .as_ref()
                .and_then(|p| p.to_str())
                .map(String::from);

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

            // ── --predict 분기: NQP 시맨틱 예측 모드 ───────────────────────
            if predict {
                // predict::run_predict 는 내부적으로 Python 서브프로세스를
                // spawn_blocking 없이 동기로 호출해도 무방 (Tokio IO 미사용).
                // 그러나 wait_with_output() 블로킹 호출이 있으므로
                // spawn_blocking 스레드에서 실행한다.
                let result =
                    tokio::task::spawn_blocking(move || predict::run_predict(&source_path))
                        .await
                        .unwrap_or_else(|e| Err(format!("스레드 패닉: {:?}", e)));

                if let Err(e) = result {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
                return Ok(());
            }

            // ── 일반 런타임 실행 분기 ────────────────────────────────────────
            if release {
                println!("🚀  릴리즈 모드 (Polars 최적화 플래그 활성화)");
                println!();
            }

            // x1zz-compiler 런타임 파이프라인 호출
            // ⚠ run_pipeline 은 Polars collect() 등 블로킹 작업을 포함하므로
            //   Tokio 비동기 컨텍스트 내부에서 직접 호출하면 "Cannot start a runtime
            //   from within a runtime" 패닉이 발생한다.
            //   → spawn_blocking 으로 Tokio 전용 블로킹 스레드 풀에서 실행한다.
            // spawn_blocking 반환타입은 Send 를 요구하므로
            // Box<dyn Error> 대신 에러를 String 으로 변환해서 반환한다.
            let result = tokio::task::spawn_blocking(move || {
                x1zz_compiler::runtime::run_pipeline(&source_path, verbose, output_csv.as_deref())
                    .map_err(|e| e.to_string())
            })
            .await
            .unwrap_or_else(|e| Err(format!("스레드 패닉: {:?}", e)));

            if let Err(e) = result {
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
                    eprintln!("IO 에러: 파일 경로를 UTF-8 문자열로 변환할 수 없습니다.");
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

                    // emit_rust 도 파일 I/O + 파싱 블로킹 작업이므로 spawn_blocking 사용
                    // Box<dyn Error> 는 Send 불가 → String 으로 변환해서 반환
                    let out_path_owned = out_path.clone();
                    let source_path_owned = source_path.clone();
                    let emit_result = tokio::task::spawn_blocking(move || {
                        x1zz_compiler::emitter::emit_rust(
                            &source_path_owned,
                            out_path_owned.as_deref(),
                        )
                        .map_err(|e| e.to_string())
                    })
                    .await
                    .unwrap_or_else(|e| Err(format!("스레드 패닉: {:?}", e)));

                    if let Err(e) = emit_result {
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

        // ── new: 새 프로젝트 생성 ─────────────────────────────────────────────
        Commands::New { name } => {
            if let Err(e) = project::create_project(&name) {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }

        // ── import: CSV → x1zz 타입 정의 + load 문 자동 생성 ─────────────────
        Commands::Import { file } => {
            if let Err(e) = schema::import_csv(&file) {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
