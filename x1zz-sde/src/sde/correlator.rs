// ============================================================
// correlator.rs — pm25 상관관계 주입
//
// 주입 수식: pm25 = a * traffic + b + N(0, σ_noise)
//
// 전략:
//   1. lf.collect() 로 DataFrame 수집 (generator는 이미 eager)
//   2. noise Series를 df.hstack_mut() 으로 실제 컬럼에 추가
//   3. df.lazy().with_column(Expr) 체인으로 pm25 오버라이드
//   4. _corr_noise 컬럼 제거 후 LazyFrame 반환
// ============================================================

use super::params::SdeConfig;
use anyhow::Result;
use polars::prelude::*;
use rand::prelude::*;
use rand::SeedableRng;
use rand_distr::{Distribution, Normal};

/// pm25 컬럼에 traffic 기반 상관관계를 주입하고 LazyFrame을 반환합니다.
pub fn inject_correlation(
    lf: LazyFrame,
    config: &SdeConfig,
    seed: u64,
) -> Result<LazyFrame> {
    let n         = config.meta.rows;
    let a         = config.correlations.pm25_traffic_a;
    let b         = config.correlations.pm25_traffic_b;
    let noise_std = config.correlations.pm25_traffic_noise;

    // ── 1. 결정적 노이즈 벡터 생성 (seed + 100) ────────────
    let mut rng = StdRng::seed_from_u64(seed + 100);
    let dist = Normal::new(0.0_f64, noise_std)?;
    let noise_vec: Vec<f64> = (0..n).map(|_| dist.sample(&mut rng)).collect();

    // ── 2. collect → hstack으로 _corr_noise 컬럼 추가 ──────
    let mut df = lf.collect()?;
    let noise_col: Column = Series::new("_corr_noise".into(), noise_vec).into();
    df.hstack_mut(&[noise_col])?;

    // ── 3. LazyFrame 체인에서 pm25 오버라이드 ──────────────
    //    _corr_noise는 df에 실제로 존재하므로 col() 참조가 반드시 성공
    let result_lf = df
        .lazy()
        .with_column(
            (lit(a) * col("traffic") + lit(b) + col("_corr_noise")).alias("pm25"),
        )
        .drop(by_name(["_corr_noise"], false, false));

    Ok(result_lf)
}
