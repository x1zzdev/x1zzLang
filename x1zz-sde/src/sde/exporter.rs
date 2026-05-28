// ============================================================
// exporter.rs — DataFrame → JSONL 출력
// ============================================================

use anyhow::Result;
use polars::prelude::*;
use std::fs::File;
use std::io::BufWriter;

/// DataFrame을 JSONL 형식으로 `output_path`에 저장합니다.
/// (write_jsonl_native의 serde_json 대안 — 현재 미사용)
pub fn write_jsonl(_df: &DataFrame, _output_path: &str) -> Result<()> {
    Ok(())
}

/// DataFrame을 Polars 네이티브 NDJSON writer로 저장합니다.
///
/// JsonWriter + JsonFormat::JsonLines 사용.
/// 파일은 BufWriter로 래핑하여 I/O 성능을 향상시킵니다.
pub fn write_jsonl_native(df: &mut DataFrame, output_path: &str) -> Result<()> {
    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);
    JsonWriter::new(&mut writer)
        .with_json_format(JsonFormat::JsonLines)
        .finish(df)?;
    Ok(())
}
