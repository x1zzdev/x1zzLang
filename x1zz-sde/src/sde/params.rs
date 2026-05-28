// ============================================================
// params.rs — sde_params.toml 파싱 및 SdeConfig 구조체
// ============================================================

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;

// ────────────────────────────────────────────────────────────
// 최상위 설정 구조체
// ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct SdeConfig {
    pub meta:         MetaConfig,
    pub columns:      HashMap<String, ColumnConfig>,
    pub correlations: CorrelationConfig,
    pub mutation:     MutationConfig,
    pub benchmark:    BenchmarkConfig,
}

// ────────────────────────────────────────────────────────────
// 하위 구조체
// ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct MetaConfig {
    pub rows:       usize,
    pub seed:       u64,
    pub output_dir: String,
}

/// 컬럼 한 개의 분포 정의
#[derive(Debug, Deserialize, Clone)]
pub struct ColumnConfig {
    /// "normal" | "poisson" | "categorical"
    pub kind:    String,
    /// Normal 분포 평균 (kind = "normal")
    pub mean:    Option<f64>,
    /// Normal 분포 표준편차 (kind = "normal")
    pub std:     Option<f64>,
    /// 클램프 하한 (optional)
    pub min:     Option<f64>,
    /// 클램프 상한 (optional)
    pub max:     Option<f64>,
    /// Poisson λ (kind = "poisson")
    pub lambda:  Option<f64>,
    /// 범주 목록 (kind = "categorical")
    pub choices: Option<Vec<String>>,
    /// 범주 가중치 (kind = "categorical")
    pub weights: Option<Vec<f64>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CorrelationConfig {
    pub pm25_traffic_a:     f64,
    pub pm25_traffic_b:     f64,
    pub pm25_traffic_noise: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MutationConfig {
    pub null_ratio:      f64,
    pub hard_case_ratio: f64,
    pub pm25_spike_val:  f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BenchmarkConfig {
    pub corr_tolerance: f64,
    pub null_tolerance: f64,
}

// ────────────────────────────────────────────────────────────
// I/O
// ────────────────────────────────────────────────────────────

/// `path`에서 TOML 파일을 읽어 `SdeConfig`로 반환합니다.
pub fn load(path: &str) -> Result<SdeConfig> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("sde_params.toml 읽기 실패: {path}"))?;
    let config: SdeConfig = toml::from_str(&raw)
        .with_context(|| "sde_params.toml TOML 파싱 오류")?;
    Ok(config)
}

// ────────────────────────────────────────────────────────────
// 유닛 테스트
// ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_sample_toml() {
        let cfg = load("sde_params.toml").expect("sde_params.toml 로드 실패");
        assert_eq!(cfg.meta.rows, 10_000);
        assert_eq!(cfg.meta.seed, 42);
        assert!(cfg.columns.contains_key("pm25"));
        assert!(cfg.columns.contains_key("traffic"));
    }
}
