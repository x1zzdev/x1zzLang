// ============================================================
// generator.rs — 독립 분포 컬럼 생성 → LazyFrame 반환
//
// 전략: 모든 RNG 샘플링(Eager)을 여기서 끝내고,
//       결과를 DataFrame → .lazy()로 변환하여 반환.
//       이후 correlator / mutator 체인은 Collect 없이 연결됨.
// ============================================================

use super::params::{ColumnConfig, SdeConfig};
use anyhow::{bail, Context, Result};
use polars::prelude::*;
use rand::prelude::*;
use rand::SeedableRng;
use rand_distr::{Distribution, Normal, Poisson};

// ────────────────────────────────────────────────────────────
// 공개 진입점
// ────────────────────────────────────────────────────────────

/// `SdeConfig`를 기반으로 독립 분포 컬럼을 생성하고
/// `LazyFrame`을 반환합니다.
///
/// 이 함수가 유일하게 RNG를 소유하며, 이후 Lazy 체인에서는
/// 미리 생성된 Vec<f64> Series를 주입하는 방식으로 처리합니다.
pub fn generate_base_dataset(config: &SdeConfig) -> Result<LazyFrame> {
    let n = config.meta.rows;
    let mut rng = StdRng::seed_from_u64(config.meta.seed);

    // 컬럼 생성 순서를 결정적으로 유지하기 위해 키를 정렬
    let mut col_names: Vec<&str> = config.columns.keys().map(|s| s.as_str()).collect();
    col_names.sort_unstable();

    let mut series_vec: Vec<Series> = Vec::with_capacity(col_names.len());

    for name in &col_names {
        let col_cfg = config
            .columns
            .get(*name)
            .expect("정렬된 키는 반드시 존재");
        let series = build_series(name, col_cfg, n, &mut rng)
            .with_context(|| format!("컬럼 '{name}' 생성 실패"))?;
        series_vec.push(series);
    }

    // Polars 0.53 파괴적 변경(Breaking Change) 대응:
    //   이전: DataFrame::new(Vec<Series>)
    //   0.53: DataFrame::new(height: usize, columns: Vec<Column>)
    //   Series → Column 변환은 From<Series> 구현체를 통해 수행
    let columns: Vec<Column> = series_vec.into_iter().map(Column::from).collect();
    let df = DataFrame::new(n, columns)
        .context("DataFrame 생성 실패")?;

    Ok(df.lazy())
}

// ────────────────────────────────────────────────────────────
// 내부 헬퍼: 분포 종류에 따라 Series 생성
// ────────────────────────────────────────────────────────────

fn build_series(
    name: &str,
    cfg: &ColumnConfig,
    n: usize,
    rng: &mut StdRng,
) -> Result<Series> {
    match cfg.kind.as_str() {
        "normal"      => build_normal(name, cfg, n, rng),
        "poisson"     => build_poisson(name, cfg, n, rng),
        "categorical" => build_categorical(name, cfg, n, rng),
        other => bail!("알 수 없는 분포 종류: '{other}'"),
    }
}

// ────────────── Normal ──────────────

fn build_normal(
    name: &str,
    cfg: &ColumnConfig,
    n: usize,
    rng: &mut StdRng,
) -> Result<Series> {
    let mean = cfg.mean.context("Normal 분포에 'mean' 필요")?;
    let std  = cfg.std.context("Normal 분포에 'std' 필요")?;

    let dist = Normal::new(mean, std)
        .with_context(|| format!("Normal({mean}, {std}) 분포 초기화 실패"))?;

    let mut vals: Vec<f64> = Vec::with_capacity(n);
    for _ in 0..n {
        let v = dist.sample(rng);
        let v = clamp(v, cfg.min, cfg.max);
        vals.push(v);
    }

    Ok(Series::new(name.into(), vals))
}

// ────────────── Poisson ──────────────

fn build_poisson(
    name: &str,
    cfg: &ColumnConfig,
    n: usize,
    rng: &mut StdRng,
) -> Result<Series> {
    let lambda = cfg.lambda.context("Poisson 분포에 'lambda' 필요")?;

    // rand_distr 0.6: Poisson<f64> — 샘플은 f64로 반환
    let dist = Poisson::<f64>::new(lambda)
        .with_context(|| format!("Poisson({lambda}) 분포 초기화 실패"))?;

    let mut vals: Vec<f64> = Vec::with_capacity(n);
    for _ in 0..n {
        let v = dist.sample(rng);
        let v = clamp(v, cfg.min, cfg.max);
        vals.push(v);
    }

    Ok(Series::new(name.into(), vals))
}

// ────────────── Categorical ──────────────

fn build_categorical(
    name: &str,
    cfg: &ColumnConfig,
    n: usize,
    rng: &mut StdRng,
) -> Result<Series> {
    let choices = cfg
        .choices
        .as_ref()
        .context("Categorical 분포에 'choices' 필요")?;
    let weights = cfg
        .weights
        .as_ref()
        .context("Categorical 분포에 'weights' 필요")?;

    if choices.len() != weights.len() {
        bail!(
            "choices({}) 와 weights({}) 길이 불일치",
            choices.len(),
            weights.len()
        );
    }

    // 가중 누적 합산을 이용한 범주 샘플링
    let cumulative = build_cumulative(weights)?;
    let mut vals: Vec<&str> = Vec::with_capacity(n);

    for _ in 0..n {
        let r: f64 = rng.random(); // [0.0, 1.0)
        let idx = cumulative
            .iter()
            .position(|&c| r < c)
            .unwrap_or(choices.len() - 1);
        vals.push(choices[idx].as_str());
    }

    Ok(Series::new(name.into(), vals))
}

// ────────────────────────────────────────────────────────────
// 유틸리티
// ────────────────────────────────────────────────────────────

/// 가중치 벡터에서 누적 확률 벡터를 생성합니다.
fn build_cumulative(weights: &[f64]) -> Result<Vec<f64>> {
    let total: f64 = weights.iter().sum();
    if total <= 0.0 {
        bail!("weights 합산이 0 이하입니다");
    }
    let mut cumulative = Vec::with_capacity(weights.len());
    let mut acc = 0.0_f64;
    for &w in weights {
        acc += w / total;
        cumulative.push(acc);
    }
    Ok(cumulative)
}

/// 선택적 min/max 범위로 값을 클램프합니다.
#[inline(always)]
fn clamp(v: f64, min: Option<f64>, max: Option<f64>) -> f64 {
    let v = if let Some(lo) = min { v.max(lo) } else { v };
    let v = if let Some(hi) = max { v.min(hi) } else { v };
    v
}

// ────────────────────────────────────────────────────────────
// 유닛 테스트
// ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sde::params;

    #[test]
    fn generate_returns_correct_row_count() {
        let cfg = params::load("sde_params.toml").expect("sde_params.toml 로드");
        let lf  = generate_base_dataset(&cfg).expect("LazyFrame 생성");
        let df  = lf.collect().expect("collect");
        assert_eq!(df.height(), cfg.meta.rows, "행 수가 일치해야 함");
    }

    #[test]
    fn generate_columns_all_present() {
        let cfg = params::load("sde_params.toml").expect("sde_params.toml 로드");
        let lf  = generate_base_dataset(&cfg).expect("LazyFrame 생성");
        let df  = lf.collect().expect("collect");
        for col in cfg.columns.keys() {
            assert!(
                df.get_column_names().contains(&col.as_str().into()),
                "컬럼 '{col}' 누락"
            );
        }
    }

    #[test]
    fn pm25_within_bounds() {
        let cfg = params::load("sde_params.toml").expect("sde_params.toml 로드");
        let lf  = generate_base_dataset(&cfg).expect("LazyFrame 생성");
        let df  = lf.collect().expect("collect");

        let pm25_cfg = cfg.columns.get("pm25").unwrap();
        let series   = df.column("pm25").unwrap();
        let max_val  = series.f64().unwrap().max().unwrap_or(0.0);
        let min_val  = series.f64().unwrap().min().unwrap_or(0.0);

        assert!(min_val >= pm25_cfg.min.unwrap_or(f64::NEG_INFINITY));
        assert!(max_val <= pm25_cfg.max.unwrap_or(f64::INFINITY));
    }

    #[test]
    fn traffic_is_non_negative() {
        let cfg = params::load("sde_params.toml").expect("sde_params.toml 로드");
        let lf  = generate_base_dataset(&cfg).expect("LazyFrame 생성");
        let df  = lf.collect().expect("collect");

        let min_val = df.column("traffic")
            .unwrap()
            .f64()
            .unwrap()
            .min()
            .unwrap_or(0.0);
        assert!(min_val >= 0.0, "Poisson 샘플은 0 이상이어야 함");
    }

    #[test]
    fn deterministic_with_same_seed() {
        let cfg = params::load("sde_params.toml").expect("sde_params.toml 로드");
        let df1 = generate_base_dataset(&cfg).unwrap().collect().unwrap();
        let df2 = generate_base_dataset(&cfg).unwrap().collect().unwrap();

        let pm25_1 = df1.column("pm25").unwrap().f64().unwrap().get(0);
        let pm25_2 = df2.column("pm25").unwrap().f64().unwrap().get(0);
        assert_eq!(pm25_1, pm25_2, "동일 시드는 동일 결과를 생성해야 함");
    }
}
