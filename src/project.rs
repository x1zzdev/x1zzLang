use anyhow::{Context, Result, bail};
use std::fs;
use std::path::Path;

/// 새 x1zzLang 프로젝트 디렉터리를 생성합니다.
///
/// 생성 구조:
/// ```text
/// {name}/
/// ├── data/
/// ├── main.xzz
/// └── x1zz.toml
/// ```
pub fn create_project(name: &str) -> Result<()> {
    let root = Path::new(name);

    // 이미 존재하면 실패
    if root.exists() {
        bail!(
            "프로젝트 생성 실패: '{}' 디렉터리가 이미 존재합니다.\n\
             다른 이름을 사용하거나 기존 디렉터리를 삭제하세요.",
            name
        );
    }

    // 루트 디렉터리 생성
    fs::create_dir_all(root.join("data"))
        .with_context(|| format!("'{}' 디렉터리 생성에 실패했습니다.", name))?;

    // main.xzz 작성 — 스펙: 주석 뒤 빈 줄 포함
    let main_xzz = "// x1zzLang Project\n\n";
    fs::write(root.join("main.xzz"), main_xzz)
        .with_context(|| "main.xzz 파일 작성에 실패했습니다.".to_string())?;

    // x1zz.toml 작성
    let toml_content = format!("[project]\nname = \"{}\"\nversion = \"0.1.0\"\n", name);
    fs::write(root.join("x1zz.toml"), toml_content)
        .with_context(|| "x1zz.toml 파일 작성에 실패했습니다.".to_string())?;

    println!("✅  프로젝트 '{}' 생성 완료!", name);
    println!();
    println!("   {}/ ", name);
    println!("   ├── data/");
    println!("   ├── main.xzz");
    println!("   └── x1zz.toml");
    println!();
    println!("   시작하려면: cd {}", name);

    Ok(())
}
