pub mod ast;
pub mod codegen;
pub mod emitter;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod runtime; // 런타임 실행 엔진 — run_pipeline() 공개
/// x1zzLang Compiler Library (v0.16)
/// 모든 서브모듈 선언 및 핵심 구조체 상위 노출
pub mod token; // Rust 코드 에밋   — emit_rust() 공개

// --- token 상위 노출 ---
pub use token::{Span, Token, TokenKind};

// --- ast 상위 노출 ---
pub use ast::{
    BinOpKind, ChartConfig, ChartType, Expr, FillNullValue, PipelineOp, PipelineSource, Program,
    Stmt, StructField,
};

// --- error 상위 노출 ---
pub use error::{CompileError, CompileResult, ErrorKind};

// --- 핵심 컴포넌트 상위 노출 ---
pub use codegen::Codegen;
pub use lexer::Lexer;
pub use parser::Parser;
