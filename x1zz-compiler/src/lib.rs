/// x1zzLang Compiler Library
///
/// 포함된 모듈:
///   - token   → x1zz-core::token 재노출 (Span, Token, TokenKind)
///   - ast     → x1zz-core::ast 재노출 (Expr, Stmt, Program, PipelineOp, ...)
///   - error   → x1zz-core::error 재노출 (CompileError, ErrorKind, ...)
///   - lexer   → Lexer (소스 문자열 → Token 배열)
///   - parser  → Parser (Token 배열 → Program AST)
///   - codegen → Codegen (AST → Polars 흐름 매핑 문자열)
///   - emitter → emit_rust (AST → 독립 Rust 소스 파일 생성)
///
/// ⚠️  런타임 실행 엔진 (run_pipeline / Polars LazyFrame)은
///      x1zz-exec 크레이트로 분리되었습니다.
///      CLI 바이너리의 Polars 의존성을 제거하기 위한 아키텍처 격리입니다.

pub mod ast;      // x1zz-core::ast 재노출
pub mod codegen;
pub mod emitter;
pub mod error;    // x1zz-core::error 재노출
pub mod lexer;
pub mod parser;
pub mod token;    // x1zz-core::token 재노출

// ── token 상위 노출 ──────────────────────────────────────────────────────────
pub use token::{Span, Token, TokenKind};

// ── ast 상위 노출 ────────────────────────────────────────────────────────────
pub use ast::{
    BinOpKind, ChartConfig, ChartType, Expr, FillNullValue, PipelineOp, PipelineSource, Program,
    Stmt, StructField,
};

// ── error 상위 노출 ──────────────────────────────────────────────────────────
pub use error::{CompileError, CompileResult, ErrorKind};

// ── 핵심 컴포넌트 상위 노출 ──────────────────────────────────────────────────
pub use codegen::Codegen;
pub use lexer::Lexer;
pub use parser::Parser;
