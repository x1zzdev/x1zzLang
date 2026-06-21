# x1zzLang — Workspace Architecture

## Binary Size Reduction Strategy

Rust 바이너리는 정적 링크됩니다.
바이너리 크기 감소는 오직 **의존성 그래프 격리**를 통해서만 달성됩니다.

---

## Final Workspace Structure

```
x1zz-lang/
├── Cargo.toml              ← workspace + x1zz CLI (루트 패키지)
├── src/                    ← x1zz CLI (경량 — Polars/Tokio 없음)
│   ├── main.rs             ← run 명령어 → x1zz-runner 서브프로세스 스폰
│   ├── cli.rs
│   ├── predict.rs
│   ├── project.rs
│   ├── schema.rs
│   └── ux.rs
│
├── x1zz-core/              ← 공유 핵심 타입 (ZERO 무거운 의존성)
│   └── src/
│       ├── lib.rs
│       ├── ast.rs          ← AST 노드 (Expr, Stmt, PipelineOp, ...)
│       ├── token.rs        ← Token, Span
│       └── error.rs        ← CompileError, ErrorKind
│
├── x1zz-compiler/          ← 컴파일러 (Polars 없음)
│   └── src/
│       ├── lib.rs
│       ├── ast.rs          ← x1zz-core::ast 재노출
│       ├── token.rs        ← x1zz-core::token 재노출
│       ├── error.rs        ← x1zz-core::error 재노출
│       ├── lexer.rs
│       ├── parser.rs
│       ├── codegen.rs
│       ├── emitter.rs
│       └── main.rs         ← 컴파일 전용 (파싱+AST 출력)
│
├── x1zz-exec/              ← 실행 엔진 (Polars 격리 크레이트)
│   └── src/
│       ├── lib.rs
│       └── runtime.rs      ← run_pipeline() — Polars LazyFrame 엔진
│
├── x1zz-runner/            ← 실행 바이너리 (CLI가 서브프로세스로 스폰)
│   └── src/
│       └── main.rs         ← x1zz-runner <file.xzz> [--verbose] [--output]
│
├── x1zz-sde/               ← 합성 데이터 생성기 (독립 — CLI와 무관)
├── x1zz-server/            ← REST API 서버 (독립 — CLI와 무관)
└── ...
```

---

## Dependency Graph

```
┌─────────────────────────────────────────────────────────────────┐
│                    x1zz (CLI binary)                            │
│  clap + indicatif + colored + csv + anyhow + encoding_rs        │
│  ✅ NO Polars  ✅ NO Tokio  ✅ NO x1zz-exec                     │
└────────────────┬────────────────────────────────────────────────┘
                 │ depends on
                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                  x1zz-compiler                                  │
│  Lexer + Parser + Codegen + Emitter                             │
│  ✅ NO Polars  ✅ NO Tokio                                       │
└────────────────┬────────────────────────────────────────────────┘
                 │ depends on
                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                    x1zz-core                                    │
│  AST + Token + Error  (serde 외 zero heavy deps)                │
└─────────────────────────────────────────────────────────────────┘

         [run 명령어: std::process::Command 서브프로세스 스폰]
x1zz CLI ──spawn──► x1zz-runner ──link──► x1zz-exec ──link──► Polars
(통신: CLI args만)

┌─────────────────────────────────────────────────────────────────┐
│                  x1zz-runner (binary)                           │
│  x1zz-runner <file.xzz> [--verbose] [--output path.csv]        │
└────────────────┬────────────────────────────────────────────────┘
                 │ depends on
                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                    x1zz-exec                                    │
│  run_pipeline() — Polars LazyFrame 런타임                       │
│  ⚠️ Polars + encoding_rs (무거운 의존성 격리)                   │
└───────┬────────────────────┬───────────────────────────────────┘
        │                    │
        ▼                    ▼
   x1zz-core          x1zz-compiler


[독립 크레이트 — CLI 의존성 그래프 외부]

x1zz-sde:    polars + rayon + x1zz-compiler (독립 바이너리)
x1zz-server: axum + tokio (독립 바이너리, x1zz-compiler 미사용)
```

---

## Crate Responsibilities

| Crate | 역할 | 무거운 의존성 | CLI 링크 |
|---|---|---|---|
| `x1zz` (CLI) | 인자 파싱, emit, import, check | 없음 | ✅ CLI 자신 |
| `x1zz-core` | AST/Token/Error 공유 타입 | 없음 (serde만) | ✅ 간접 |
| `x1zz-compiler` | Lexer/Parser/Codegen/Emitter | 없음 | ✅ emit 명령어 |
| `x1zz-exec` | Polars 실행 엔진 | **Polars, encoding_rs** | ❌ 없음 |
| `x1zz-runner` | 실행 바이너리 | x1zz-exec 통해 간접 | ❌ 없음 |
| `x1zz-sde` | 합성 데이터 생성기 | polars, rayon | ❌ 없음 |
| `x1zz-server` | REST API 서버 | axum, tokio | ❌ 없음 |

---

## Execution Boundary (OPTION A — subprocess)

```
x1zz run file.xzz
    │
    ├─ find_runner() → 같은 디렉토리의 x1zz-runner.exe 또는 PATH
    │
    └─ std::process::Command::new("x1zz-runner")
           .arg("file.xzz")
           .arg("--verbose")      // optional
           .arg("--output")       // optional
           .arg("result.csv")
           .status()
```

**통신 프로토콜:**  
- 입력: CLI arguments만 (JSON stdin 불필요)  
- 출력: x1zz-runner의 stdout/stderr 그대로 전달  
- 종료 코드: x1zz-runner의 exit code 전파

---

## Migration Summary

### Before (의존성 체인 — Polars가 CLI에 포함됨)
```
x1zz CLI → x1zz-compiler → polars (🚫 CLI 바이너리에 Polars 링크됨)
x1zz CLI → tokio (🚫 비동기 런타임 링크됨)
```

### After (의존성 격리 — Polars가 CLI에서 제거됨)
```
x1zz CLI → x1zz-compiler → x1zz-core → serde
x1zz-runner → x1zz-exec → polars (✅ 분리된 바이너리)
```

### Binary Size Impact (예상)
| Binary | Before | After | 차이 |
|---|---|---|---|
| `x1zz` (CLI) | ~35MB+ (Polars 포함) | ~2-5MB | **~85% 감소** |
| `x1zz-runner` | N/A | ~30MB+ | 실행 엔진 담당 |

---

## Build Commands

```bash
# 전체 워크스페이스 빌드
cargo build --release

# CLI 단독 빌드 (경량)
cargo build -p x1zz --release

# 실행 엔진 단독 빌드 (Polars 포함)
cargo build -p x1zz-runner --release

# 배포 시 두 바이너리를 같은 디렉토리에 배치
# x1zz.exe + x1zz-runner.exe
```

---

## Rules

1. `x1zz` (CLI) 의 `[dependencies]` 에 절대 포함하면 안 되는 크레이트:
   - `polars`, `polars-*`
   - `tokio`
   - `rayon`
   - `x1zz-exec`
   - `x1zz-runner`

2. `x1zz-exec` 는 CLI 의존성 그래프 외부에서만 사용한다.

3. `x1zz-compiler` 는 Polars를 의존하지 않는다 (파싱/코드생성만).

4. 새로운 실행 로직은 반드시 `x1zz-exec` 에 추가한다.
