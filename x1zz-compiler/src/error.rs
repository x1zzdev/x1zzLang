/// x1zzLang - 컴파일러 에러 타입 정의

use crate::token::Span;

/// 컴파일 에러 종류
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    /// 렉서: 알 수 없는 문자
    UnexpectedChar(char),
    /// 파서: 예상치 못한 토큰
    UnexpectedToken(String),
    /// 파서: 예상 토큰 미등장
    ExpectedToken(String),
    /// 코드젠: 미선언 타입 참조
    UndeclaredType(String),
    /// 기타
    Other(String),
}

/// 컴파일 에러 구조체
#[derive(Debug, Clone, PartialEq)]
pub struct CompileError {
    pub kind: ErrorKind,
    pub span: Span,
    pub message: String,
}

impl CompileError {
    pub fn new(kind: ErrorKind, span: Span, message: impl Into<String>) -> Self {
        CompileError {
            kind,
            span,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[x1zz Error] {}:{} — {}",
            self.span.line, self.span.col, self.message
        )
    }
}

impl std::error::Error for CompileError {}

/// 컴파일 결과 타입 별칭
pub type CompileResult<T> = Result<T, CompileError>;
