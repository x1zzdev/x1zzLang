/// x1zzLang - 렉서 (Peekable<Chars> 상태 머신 완전 구현)
///
/// 모든 필드를 실제로 사용하여 dead_code 경고 없음:
///   source  - byte 오프셋 경계 확인용 (is_at_end)
///   chars   - 상태 머신 이터레이터
///   pos     - 현재 바이트 오프셋 (UTF-8 len_utf8 누적)
///   line/col- 소스 위치 추적

use crate::error::{CompileError, CompileResult, ErrorKind};
use crate::token::{Span, Token, TokenKind};

pub struct Lexer<'src> {
    source: &'src str,
    chars: std::iter::Peekable<std::str::Chars<'src>>,
    pos: usize,
    line: usize,
    col: usize,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Self {
        Lexer {
            source,
            chars: source.chars().peekable(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    // ── 기본 헬퍼 ────────────────────────────────────────────────────────────

    /// 이터레이터를 끝까지 소비했는지 확인 (source 필드 참조)
    fn is_at_end(&self) -> bool {
        self.pos >= self.source.len()
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    /// 한 문자를 소비하고 pos / line / col 갱신
    fn advance(&mut self) -> Option<char> {
        match self.chars.next() {
            Some(c) => {
                self.pos += c.len_utf8();
                if c == '\n' {
                    self.line += 1;
                    self.col = 1;
                } else {
                    self.col += 1;
                }
                Some(c)
            }
            None => None,
        }
    }

    fn span(&self) -> Span {
        Span::new(self.line, self.col)
    }

    // ── 문자열 리터럴 ────────────────────────────────────────────────────────

    /// 여는 '"' 이미 소비된 상태에서 호출
    fn read_string(&mut self, open_span: Span) -> CompileResult<TokenKind> {
        let mut s = String::new();
        loop {
            match self.advance() {
                Some('"') => break,
                Some('\\') => match self.advance() {
                    Some('n')  => s.push('\n'),
                    Some('t')  => s.push('\t'),
                    Some('\\') => s.push('\\'),
                    Some('"')  => s.push('"'),
                    Some(c)    => s.push(c),
                    None => {
                        return Err(CompileError::new(
                            ErrorKind::UnexpectedToken("EOF in string escape".into()),
                            open_span,
                            "문자열 이스케이프 처리 중 파일 끝",
                        ));
                    }
                },
                Some(c) => s.push(c),
                None => {
                    return Err(CompileError::new(
                        ErrorKind::UnexpectedToken("Unterminated string".into()),
                        open_span,
                        "닫는 '\"' 없이 파일이 끝남",
                    ));
                }
            }
        }
        Ok(TokenKind::StringLit(s))
    }

    // ── 숫자 리터럴 ──────────────────────────────────────────────────────────

    /// 첫 번째 자리(first)는 이미 소비된 상태
    fn read_number(&mut self, first: char) -> TokenKind {
        let mut buf = String::new();
        buf.push(first);

        // 정수 부분
        while self.peek().map_or(false, |c| c.is_ascii_digit()) {
            buf.push(self.advance().unwrap());
        }

        // 소수점 + 소수 부분
        if self.peek() == Some('.') {
            self.advance();   // '.' 소비
            buf.push('.');
            while self.peek().map_or(false, |c| c.is_ascii_digit()) {
                buf.push(self.advance().unwrap());
            }
            return TokenKind::FloatLit(buf.parse().unwrap_or(0.0));
        }

        TokenKind::IntLit(buf.parse().unwrap_or(0))
    }

    // ── 식별자 · 키워드 ──────────────────────────────────────────────────────

    fn read_ident(&mut self, first: char) -> TokenKind {
        let mut buf = String::new();
        buf.push(first);
        while self.peek().map_or(false, |c| c.is_alphanumeric() || c == '_') {
            buf.push(self.advance().unwrap());
        }
        Self::keyword_or_ident(buf)
    }

    fn keyword_or_ident(s: String) -> TokenKind {
        match s.as_str() {
            "type"   => TokenKind::Type,
            "load"   => TokenKind::Load,
            "filter" => TokenKind::Filter,
            "select" => TokenKind::Select,
            "count"  => TokenKind::Count,
            "v"      => TokenKind::V,
            "mut"    => TokenKind::Mut,
            "Option" => TokenKind::OptionKw,
            _        => TokenKind::Ident(s),
        }
    }

    // ── 메인 상태 머신 ────────────────────────────────────────────────────────

    /// 다음 Token을 하나 반환 (상태 머신 핵심)
    pub fn next_token(&mut self) -> CompileResult<Token> {
        // 공백 건너뛰기
        while self.peek().map_or(false, |c| c.is_whitespace()) {
            self.advance();
        }

        // 파일 끝
        if self.is_at_end() {
            return Ok(Token::new(TokenKind::Eof, self.span()));
        }

        let span = self.span();
        let ch = match self.advance() {
            Some(c) => c,
            None    => return Ok(Token::new(TokenKind::Eof, span)),
        };

        let kind = match ch {
            // ── 주석 ──────────────────────────────────────────────────
            '/' if self.peek() == Some('/') => {
                // 줄 끝까지 소비 후 재귀 호출
                while self.peek().map_or(false, |c| c != '\n') {
                    self.advance();
                }
                return self.next_token();
            }
            '/' => TokenKind::Slash,

            // ── 문자열 ────────────────────────────────────────────────
            '"' => self.read_string(span.clone())?,

            // ── 두 문자 연산자 ─────────────────────────────────────────
            '|' if self.peek() == Some('>') => {
                self.advance();
                TokenKind::Pipeline
            }
            ':' if self.peek() == Some(':') => {
                self.advance();
                TokenKind::TypeAssign
            }
            '=' if self.peek() == Some('=') => {
                self.advance();
                TokenKind::EqEq
            }
            '!' if self.peek() == Some('=') => {
                self.advance();
                TokenKind::NotEq
            }
            '<' if self.peek() == Some('=') => {
                self.advance();
                TokenKind::LtEq
            }
            '>' if self.peek() == Some('=') => {
                self.advance();
                TokenKind::GtEq
            }

            // ── 단일 문자 연산자 ───────────────────────────────────────
            '=' => TokenKind::Assign,
            '<' => TokenKind::Lt,
            '>' => TokenKind::Gt,
            '+' => TokenKind::Plus,
            '*' => TokenKind::Star,
            '!' => TokenKind::Bang,
            '.' => TokenKind::Dot,
            ':' => TokenKind::Colon,

            // ── 음수 또는 Minus ────────────────────────────────────────
            '-' if self.peek().map_or(false, |c| c.is_ascii_digit()) => {
                let digit = self.advance().unwrap();
                match self.read_number(digit) {
                    TokenKind::IntLit(n)   => TokenKind::IntLit(-n),
                    TokenKind::FloatLit(f) => TokenKind::FloatLit(-f),
                    other                  => other,
                }
            }
            '-' => TokenKind::Minus,

            // ── 구분자 ────────────────────────────────────────────────
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            ',' => TokenKind::Comma,
            ';' => TokenKind::Semicolon,

            // ── 숫자 ───────────────────────────────────────────────────
            c if c.is_ascii_digit() => self.read_number(c),

            // ── 식별자 / 키워드 ────────────────────────────────────────
            c if c.is_alphabetic() || c == '_' => self.read_ident(c),

            // ── 알 수 없는 문자 ────────────────────────────────────────
            other => {
                return Err(CompileError::new(
                    ErrorKind::UnexpectedChar(other),
                    span,
                    format!("예상치 못한 문자: '{}'", other),
                ));
            }
        };

        Ok(Token::new(kind, span))
    }

    /// 소스 전체를 토크나이징하여 Vec<Token> 반환
    pub fn tokenize(&mut self) -> CompileResult<Vec<Token>> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token()?;
            let done = matches!(tok.kind, TokenKind::Eof) || self.is_at_end();
            tokens.push(tok);
            if done {
                break;
            }
        }
        Ok(tokens)
    }
}
