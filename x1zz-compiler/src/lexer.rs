/// x1zzLang - 렉서 (완전 구현)
///
/// 지원 문법 요소:
///   키워드: type load filter select count v mut Option
///   연산자: |> :: = == != < > <= >= + - * / !
///   구분자: { } ( ) [ ] , ; :
///   리터럴: "string", 42, 3.14, -42, -3.14
///   주석  : // 한 줄 주석

use crate::error::{CompileError, CompileResult, ErrorKind};
use crate::token::{Span, Token, TokenKind};

pub struct Lexer<'src> {
    source: &'src str,
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Self {
        Lexer {
            source,
            chars: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    // ── 기본 헬퍼 ────────────────────────────────────────────────────────────

    fn current(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }

    fn peek(&self) -> Option<char> {
        self.peek_at(1)
    }

    /// 현재 문자를 소비하고 줄/열 추적 갱신
    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.get(self.pos).copied();
        if let Some(c) = ch {
            self.pos += 1;
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        ch
    }

    fn span(&self) -> Span {
        Span::new(self.line, self.col)
    }

    // ── 공백 · 주석 스킵 ─────────────────────────────────────────────────────

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            // 공백 건너뛰기
            while self.current().map_or(false, |c| c.is_whitespace()) {
                self.advance();
            }
            // 한 줄 주석 건너뛰기: `//`
            if self.current() == Some('/') && self.peek() == Some('/') {
                while self.current().map_or(false, |c| c != '\n') {
                    self.advance();
                }
            } else {
                break;
            }
        }
    }

    // ── 문자열 리터럴 ────────────────────────────────────────────────────────

    fn read_string(&mut self) -> CompileResult<String> {
        let span = self.span();
        self.advance(); // 여는 '"' 소비
        let mut s = String::new();
        loop {
            match self.current() {
                Some('"') => {
                    self.advance(); // 닫는 '"' 소비
                    break;
                }
                Some('\\') => {
                    self.advance();
                    match self.advance() {
                        Some('n')  => s.push('\n'),
                        Some('t')  => s.push('\t'),
                        Some('\\') => s.push('\\'),
                        Some('"')  => s.push('"'),
                        Some(c)    => s.push(c),
                        None       => {
                            return Err(CompileError::new(
                                ErrorKind::UnexpectedToken("문자열 끝에서 탈출 문자".into()),
                                span,
                                "문자열 리터럴이 끝나기 전에 파일이 종료됨",
                            ));
                        }
                    }
                }
                Some(c) => {
                    s.push(c);
                    self.advance();
                }
                None => {
                    return Err(CompileError::new(
                        ErrorKind::UnexpectedToken("Unterminated string".into()),
                        span,
                        "문자열 리터럴이 닫히지 않음 (\" 누락)",
                    ));
                }
            }
        }
        Ok(s)
    }

    // ── 숫자 리터럴 ──────────────────────────────────────────────────────────

    /// 부호 없는 숫자를 읽어 IntLit 또는 FloatLit 반환
    fn read_unsigned_number(&mut self) -> TokenKind {
        let mut buf = String::new();
        let mut is_float = false;

        while let Some(c) = self.current() {
            if c.is_ascii_digit() {
                buf.push(c);
                self.advance();
            } else if c == '.' && self.peek().map_or(false, |p| p.is_ascii_digit()) {
                // 소수점 (다음 문자도 숫자인 경우만 소수로 처리)
                is_float = true;
                buf.push(c);
                self.advance();
            } else {
                break;
            }
        }

        if is_float {
            TokenKind::FloatLit(buf.parse().unwrap_or(0.0))
        } else {
            TokenKind::IntLit(buf.parse().unwrap_or(0))
        }
    }

    // ── 식별자 · 키워드 ──────────────────────────────────────────────────────

    fn read_ident(&mut self) -> String {
        let mut buf = String::new();
        while let Some(c) = self.current() {
            if c.is_alphanumeric() || c == '_' {
                buf.push(c);
                self.advance();
            } else {
                break;
            }
        }
        buf
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

    // ── 메인 토크나이저 ───────────────────────────────────────────────────────

    pub fn tokenize(&mut self) -> CompileResult<Vec<Token>> {
        let mut tokens: Vec<Token> = Vec::new();

        loop {
            self.skip_whitespace_and_comments();
            let span = self.span();

            let ch = match self.current() {
                None => {
                    tokens.push(Token::new(TokenKind::Eof, span));
                    break;
                }
                Some(c) => c,
            };

            let kind = match ch {
                // ── 문자열 리터럴 ───────────────────────────────────────
                '"' => TokenKind::StringLit(self.read_string()?),

                // ── 두 문자 연산자 우선 처리 ────────────────────────────
                '|' if self.peek() == Some('>') => {
                    self.advance(); self.advance();
                    TokenKind::Pipeline
                }
                ':' if self.peek() == Some(':') => {
                    self.advance(); self.advance();
                    TokenKind::TypeAssign
                }
                '=' if self.peek() == Some('=') => {
                    self.advance(); self.advance();
                    TokenKind::EqEq
                }
                '!' if self.peek() == Some('=') => {
                    self.advance(); self.advance();
                    TokenKind::NotEq
                }
                '<' if self.peek() == Some('=') => {
                    self.advance(); self.advance();
                    TokenKind::LtEq
                }
                '>' if self.peek() == Some('=') => {
                    self.advance(); self.advance();
                    TokenKind::GtEq
                }

                // ── 단일 문자 연산자 ────────────────────────────────────
                '=' => { self.advance(); TokenKind::Assign }
                '<' => { self.advance(); TokenKind::Lt }
                '>' => { self.advance(); TokenKind::Gt }
                '+' => { self.advance(); TokenKind::Plus }
                '*' => { self.advance(); TokenKind::Star }
                '/' => { self.advance(); TokenKind::Slash }
                '!' => { self.advance(); TokenKind::Bang }
                '.' => { self.advance(); TokenKind::Dot }

                // ── 부호 있는 음수 또는 Minus 단독 ─────────────────────
                '-' if self.peek().map_or(false, |p| p.is_ascii_digit()) => {
                    self.advance(); // '-' 소비
                    match self.read_unsigned_number() {
                        TokenKind::IntLit(n)  => TokenKind::IntLit(-n),
                        TokenKind::FloatLit(f) => TokenKind::FloatLit(-f),
                        other                  => other,
                    }
                }
                '-' => { self.advance(); TokenKind::Minus }

                // ── 구분자 ─────────────────────────────────────────────
                ':' => { self.advance(); TokenKind::Colon }
                '{' => { self.advance(); TokenKind::LBrace }
                '}' => { self.advance(); TokenKind::RBrace }
                '(' => { self.advance(); TokenKind::LParen }
                ')' => { self.advance(); TokenKind::RParen }
                '[' => { self.advance(); TokenKind::LBracket }
                ']' => { self.advance(); TokenKind::RBracket }
                ',' => { self.advance(); TokenKind::Comma }
                ';' => { self.advance(); TokenKind::Semicolon }

                // ── 숫자 리터럴 ────────────────────────────────────────
                c if c.is_ascii_digit() => self.read_unsigned_number(),

                // ── 식별자 / 키워드 ────────────────────────────────────
                c if c.is_alphabetic() || c == '_' => {
                    let ident = self.read_ident();
                    Self::keyword_or_ident(ident)
                }

                // ── 예상치 못한 문자 → 에러 ────────────────────────────
                other => {
                    return Err(CompileError::new(
                        ErrorKind::UnexpectedChar(other),
                        span,
                        format!("예상치 못한 문자: '{}'", other),
                    ));
                }
            };

            tokens.push(Token::new(kind, span));
        }

        Ok(tokens)
    }
}
