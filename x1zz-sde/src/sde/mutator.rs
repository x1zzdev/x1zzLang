// ============================================================
// mutator.rs — LazyFrame 기반 null/hard-case/alias 주입
//
// 전략: RNG 값을 Vec<f64>로 미리 생성 → Series로 주입
//       Polars when/then/otherwise Expr으로 변형 → LazyFrame 반환
//       collect() 호출 없음
// ============================================================

use super::params::SdeConfig;
use anyhow::Result;
use polars::prelude::*;
use rand::prelude::*;
use rand::SeedableRng;

/// pm25 컬럼에 null_ratio 비율만큼 null 값을 주입합니다.
pub fn inject_nulls(lf: LazyFrame, config: &SdeConfig, seed: u64) -> Result<LazyFrame> {
    let n          = config.meta.rows;
    let null_ratio = config.mutation.null_ratio;

    // 결정적 랜덤 벡터 (seed + 200)
    let mut rng = StdRng::seed_from_u64(seed + 200);
    let rand_vec: Vec<f64> = (0..n).map(|_| rng.random::<f64>()).collect();
    let rand_series = Series::new("_rand_null".into(), rand_vec);

    let lf = lf
        .with_column(lit(rand_series))
        .with_column(
            when(col("_rand_null").lt(lit(null_ratio)))
                .then(lit(NULL).cast(DataType::Float64))
                .otherwise(col("pm25"))
                .alias("pm25"),
        )
        .drop(by_name(["_rand_null"], false, false));

    Ok(lf)
}

/// pm25 컬럼에 hard_case_ratio 비율만큼 황사 스파이크 극단값을 주입합니다.
pub fn inject_hard_cases(
    lf: LazyFrame,
    config: &SdeConfig,
    seed: u64,
) -> Result<LazyFrame> {
    let n          = config.meta.rows;
    let hard_ratio = config.mutation.hard_case_ratio;
    let spike_val  = config.mutation.pm25_spike_val;

    // 결정적 랜덤 벡터 (seed + 300)
    let mut rng = StdRng::seed_from_u64(seed + 300);
    let rand_vec: Vec<f64> = (0..n).map(|_| rng.random::<f64>()).collect();
    let rand_series = Series::new("_rand_hard".into(), rand_vec);

    let lf = lf
        .with_column(lit(rand_series))
        .with_column(
            when(col("_rand_hard").lt(lit(hard_ratio)))
                .then(lit(spike_val))
                .otherwise(col("pm25"))
                .alias("pm25"),
        )
        .drop(by_name(["_rand_hard"], false, false));

    Ok(lf)
}

/// 컬럼 alias 적용 (현재 config에 alias_map 없으므로 pass-through)
pub fn apply_aliases(lf: LazyFrame, _config: &SdeConfig) -> Result<LazyFrame> {
    Ok(lf)
}
