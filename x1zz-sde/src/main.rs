// ============================================================
// main.rs — x1zz-sde CLI 진입점
//
// 실행: cargo run -p x1zz-sde
// 릴리즈: cargo run -p x1zz-sde --release
//
// 파이프라인 (LazyFrame-First):
//   params::load → generate → correlate → mutate
//     → .collect() (단 1회) → export → benchmark
// ============================================================

use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use x1zz_sde::sde;

fn main() -> Result<()> {
    // ── 1. 파라미터 로드 ──────────────────────────────────────
    let pb = spinner("📄 sde_params.toml 로드 중...");
    let config = sde::params::load("x1zz-sde/sde_params.toml")?;
    pb.finish_with_message(format!(
        "✅ 파라미터 로드 완료 (rows={}, seed={})",
        config.meta.rows, config.meta.seed
    ));

    // ── 2. 기저 데이터 생성 (RNG → LazyFrame) ─────────────────
    let pb = spinner("🎲 기저 데이터 생성 중...");
    let lf = sde::generate_base_dataset(&config)?;
    pb.finish_with_message("✅ 기저 LazyFrame 생성 완료");

    // ── 3. 상관관계 주입 (LazyFrame 체인, collect 없음) ────────
    let pb = spinner("🔗 상관관계 주입 중...");
    let lf = sde::inject_correlation(lf, &config, config.meta.seed)?;
    pb.finish_with_message("✅ 상관관계 주입 완료");

    // ── 4. 변형 주입 (LazyFrame 체인, collect 없음) ────────────
    let pb = spinner("🔧 null/hard-case 주입 중...");
    let lf = sde::inject_nulls(lf, &config, config.meta.seed)?;
    let lf = sde::inject_hard_cases(lf, &config, config.meta.seed)?;
    let lf = sde::apply_aliases(lf, &config)?;
    pb.finish_with_message("✅ 변형 주입 완료");

    // ── 5. 유일한 collect() 호출 ──────────────────────────────
    let pb = spinner("⚙️  쿼리 플랜 최적화 및 실행 중...");
    let mut df = lf.collect()?;
    pb.finish_with_message(format!(
        "✅ DataFrame 수집 완료 ({}행 × {}열)",
        df.height(),
        df.width()
    ));

    // ── 6. JSONL 출력 ──────────────────────────────────────────
    let output_path = format!("{}/train.jsonl", config.meta.output_dir);
    let pb = spinner(format!("💾 JSONL 저장 중 → {output_path}"));
    std::fs::create_dir_all(&config.meta.output_dir)?;
    sde::write_jsonl_native(&mut df, &output_path)?;
    pb.finish_with_message(format!("✅ JSONL 저장 완료: {output_path}"));

    // ── 7. 품질 검증 (R² 기반) ────────────────────────────────
    let pb = spinner("🔬 품질 검증 중...");
    let report = sde::validate(&df, &config)?;
    pb.finish_with_message("✅ 품질 검증 완료");

    // ── 최종 보고서 출력 ──────────────────────────────────────
    println!();
    println!("╔══════════════════════════════════════╗");
    println!("║       x1zz-SDE 생성 보고서            ║");
    println!("╠══════════════════════════════════════╣");
    println!("║ 생성 행 수        : {:>8}          ║", report.row_count);
    println!("║ 이론 Pearson r    : {:>+8.4}          ║", report.theory_pearson_r);
    println!("║ 실측 Pearson r    : {:>+8.4}          ║", report.actual_pearson_r);
    println!("║ 상관계수 오차     : {:>7.2}%          ║", report.corr_error_pct);
    println!("║ pm25 null 비율    : {:>7.2}%          ║", report.null_ratio_pm25 * 100.0);
    println!("║ 품질 검증         : {:>8}          ║", if report.passed { "PASS ✅" } else { "STUB ⚠️ " });
    println!("╚══════════════════════════════════════╝");

    Ok(())
}

// ────────────────────────────────────────────────────────────
// 유틸리티
// ────────────────────────────────────────────────────────────

fn spinner(msg: impl Into<String>) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message(msg.into());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}
