// x1zz-compiler/src/main.rs  (v0.17)
//
// 컴파일러 직접 실행 엔트리포인트.
//
// 지원되는 두 가지 입력 모드:
//   1. .xzz 소스 파일  →  기존 컴파일+런타임 파이프라인
//   2. .csv 데이터 파일 →  벤치마크 파이프라인 .xzz 자동 생성 후 즉시 실행
//
// 사용 예:
//   cargo run -p x1zz-compiler -- examples/poc_script.xzz
//   cargo run --release -p x1zz-compiler -- benches/data/scale_large.csv

use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let input_path = args
        .get(1)
        .map(String::as_str)
        .unwrap_or("examples/poc_script.xzz");

    // ── CSV 직접 입력 감지: 벤치마크 파이프라인 자동 생성 ───────────────────────
    // --verbose / -v 플래그 감지
    let verbose = args.iter().any(|a| a == "--verbose" || a == "-v");

    if input_path.to_lowercase().ends_with(".csv") {
        run_csv_benchmark(input_path, verbose);
    } else {
        if let Err(e) = x1zz_compiler::runtime::run_pipeline(input_path, verbose, None) {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

/// CSV 경로를 받아 벤치마크용 .xzz 스크립트를 임시 파일로 생성한 뒤
/// run_pipeline()으로 실행하고 임시 파일을 정리한다.
fn run_csv_benchmark(csv_path: &str, verbose: bool) {
    // 크로스 플랫폼: 백슬래시 → 슬래시
    let posix_path = csv_path.replace('\\', "/");

    let xzz_source = format!(
        r#"// x1zzLang Benchmark Pipeline — auto-generated from CSV input
//
// Pipeline stages matching the Pandas baseline (pandas_pipeline.py):
//   P2: dropNull(pm10) | filter pm10<120 & pm25>10
//   P3: groupBy(station) -> sum(pm10)
//   P4: groupBy(station) -> mean(pm10) -> sort desc -> take(10)
//   P7: fillNull(pm25,0) | filter pm10>50 -> groupBy -> count -> top5

type AirQuality = {{
  date: string,
  station: string,
  pm10: Option<float>,
  pm25: Option<float>,
}};

v raw = load("{posix_path}") :: AirQuality
  |> select([date, station, pm10, pm25]);

v cleaned = raw
  |> dropNull("pm10")
  |> filter(col("pm10") < 120)
  |> filter(col("pm25") > 10);

v by_station = cleaned
  |> groupBy("station")
  |> sum("pm10");

v top10_mean = cleaned
  |> groupBy("station")
  |> mean("pm10")
  |> orderBy("pm10", desc: true)
  |> take(10);

v filled = raw
  |> fillNull("pm25", 0)
  |> filter(col("pm10") > 50)
  |> groupBy("station")
  |> count("pm25")
  |> orderBy("pm25", desc: true)
  |> take(5);
"#,
        posix_path = posix_path
    );

    // 임시 .xzz 파일 경로: 원본 CSV와 동일한 디렉터리에 생성
    let tmp_xzz_path = if let Some(parent) = Path::new(csv_path).parent() {
        let stem = Path::new(csv_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("benchmark");
        parent
            .join(format!("_{}_bench.xzz", stem))
            .to_string_lossy()
            .to_string()
    } else {
        format!("_{}_bench.xzz", csv_path)
    };

    if let Err(e) = std::fs::write(&tmp_xzz_path, &xzz_source) {
        eprintln!("[ERROR] 임시 .xzz 파일 생성 실패: {} — {}", tmp_xzz_path, e);
        std::process::exit(1);
    }

    let result = x1zz_compiler::runtime::run_pipeline(&tmp_xzz_path, verbose, None);

    // 임시 파일 정리 (결과와 무관하게 삭제)
    let _ = std::fs::remove_file(&tmp_xzz_path);

    if let Err(e) = result {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
