// ============================================================
// benchmark.rs — 생성 데이터 품질 검증 (Pearson r 기반)
//
// 검증 목표:
//   1. 실제 Pearson r(pm25, traffic) vs 이론값 오차 ≤ corr_tolerance
//   2. pm25 null 비율 오차 ≤ null_tolerance
//   3. 10,000행 생성 확인
// ============================================================

use super::params::SdeConfig;
use anyhow::Result;
use polars::prelude::*;

/// 품질 검증 보고서
#[derive(Debug)]
pub struct QualityReport {
    pub row_count:        usize,
    pub actual_pearson_r: f64,
    pub theory_pearson_r: f64,
    pub corr_error_pct:   f64,   // |actual - theory| / |theory| × 100
    pub null_ratio_pm25:  f64,
    pub passed:           bool,
}

/// 생성된 DataFrame의 품질을 검증하고 `QualityReport`를 반환합니다.
///
/// # 조건
///   - `corr_error_pct / 100` ≤ `config.benchmark.corr_tolerance`
///   - `|null_ratio - expected_null_ratio|` ≤ `config.benchmark.null_tolerance`
///   - `row_count` == `config.meta.rows`
pub fn validate(df: &DataFrame, config: &SdeConfig) -> Result<QualityReport> {
    let pm25    = df.column("pm25")?;
    let row_count = df.height();

    // ── 1. pm25 null 비율 ──────────────────────────────────
    let null_ratio = pm25.null_count() as f64 / pm25.len() as f64;

    // ── 2. 이론 Pearson r ──────────────────────────────────
    let theory_r = compute_theoretical_pearson_r(config);

    // ── 3. 실측 Pearson r (null + spike 행 제거 후 수동 계산) ─────
    let spike_val = config.mutation.pm25_spike_val;
    let actual_r = manual_pearson_r(df, spike_val)?;

    // ── 4. 상관계수 오차 백분율 ────────────────────────────
    let corr_error_pct = if theory_r.abs() > 1e-12 {
        (actual_r - theory_r).abs() / theory_r.abs() * 100.0
    } else {
        actual_r.abs() * 100.0
    };

    // ── 5. passed 판정 ─────────────────────────────────────
    let null_expected = config.mutation.null_ratio;
    let corr_ok = (corr_error_pct / 100.0) <= config.benchmark.corr_tolerance;
    let null_ok = (null_ratio - null_expected).abs() <= config.benchmark.null_tolerance;
    let row_ok  = row_count == config.meta.rows;

    Ok(QualityReport {
        row_count,
        actual_pearson_r: actual_r,
        theory_pearson_r: theory_r,
        corr_error_pct,
        null_ratio_pm25:  null_ratio,
        passed: corr_ok && null_ok && row_ok,
    })
}

/// pm25, traffic 컬럼에서 Pearson r을 수동으로 계산합니다.
/// pm25 null 행 및 spike 행(pm25 >= spike_threshold)은 제외합니다.
fn manual_pearson_r(df: &DataFrame, spike_threshold: f64) -> Result<f64> {
    let xf = df.column("pm25")?.cast(&DataType::Float64)?;
    let yf = df.column("traffic")?.cast(&DataType::Float64)?;
    let xc = xf.f64()?;
    let yc = yf.f64()?;

    // null이 없고, spike가 아닌 쌍만 수집
    let (xs, ys): (Vec<f64>, Vec<f64>) = xc
        .iter()
        .zip(yc.iter())
        .filter_map(|(a, b)| match (a, b) {
            (Some(x), Some(y)) if x < spike_threshold => Some((x, y)),
            _ => None,
        })
        .unzip();

    let n = xs.len();
    if n < 2 {
        return Ok(0.0);
    }

    let mx: f64 = xs.iter().sum::<f64>() / n as f64;
    let my: f64 = ys.iter().sum::<f64>() / n as f64;

    let mut cov = 0.0_f64;
    let mut vx  = 0.0_f64;
    let mut vy  = 0.0_f64;

    for (&xi, &yi) in xs.iter().zip(ys.iter()) {
        let dx = xi - mx;
        let dy = yi - my;
        cov += dx * dy;
        vx  += dx * dx;
        vy  += dy * dy;
    }

    if vx < 1e-12 || vy < 1e-12 {
        return Ok(0.0);
    }

    Ok(cov / (vx * vy).sqrt())
}

/// 파라미터로부터 이론적 Pearson r을 계산합니다.
///
/// 수식:
///   λ = traffic.lambda,  a = pm25_traffic_a,  σ = pm25_traffic_noise
///   Var_traffic = λ
///   Cov(pm25, traffic) = a · λ
///   Var_pm25 = a² · λ + σ²
///   r = Cov / sqrt(Var_traffic · Var_pm25)
pub fn compute_theoretical_pearson_r(config: &SdeConfig) -> f64 {
    let lambda = config
        .columns
        .get("traffic")
        .and_then(|c| c.lambda)
        .unwrap_or(3200.0);

    let a       = config.correlations.pm25_traffic_a;
    let sigma_n = config.correlations.pm25_traffic_noise;

    let cov      = a * lambda;
    let var_pm25 = a * a * lambda + sigma_n * sigma_n;
    let r        = cov / (lambda * var_pm25).sqrt();

    r
}
