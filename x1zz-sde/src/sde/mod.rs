// ============================================================
// sde/mod.rs — SDE 모듈 트리 선언 및 공개 재수출
//
// 파이프라인 실행 순서:
//   params::load()
//     → generator::generate_base_dataset()   // RNG, LazyFrame 반환
//     → correlator::inject_correlation()      // LazyFrame 체인
//     → mutator::inject_nulls()               // LazyFrame 체인
//     → mutator::inject_hard_cases()          // LazyFrame 체인
//     → mutator::apply_aliases()              // LazyFrame 체인
//     → .collect()                            // 유일한 collect 호출
//     → exporter::write_jsonl_native()        // JSONL 출력
//     → benchmark::validate()                 // R² 품질 검증
// ============================================================

// ── 현재 구현 완료 ──────────────────────────────────────────
pub mod params;
pub mod generator;

// ── Week 2 구현 예정 ────────────────────────────────────────
pub mod correlator;
pub mod mutator;
pub mod exporter;
pub mod benchmark;

// ────────────────────────────────────────────────────────────
// 공개 재수출 (외부 크레이트 및 main.rs에서 편리하게 import)
// ────────────────────────────────────────────────────────────

pub use params::{SdeConfig, load};
pub use generator::generate_base_dataset;
pub use correlator::inject_correlation;
pub use mutator::{inject_nulls, inject_hard_cases, apply_aliases};
pub use exporter::write_jsonl_native;
pub use benchmark::{validate, QualityReport};
