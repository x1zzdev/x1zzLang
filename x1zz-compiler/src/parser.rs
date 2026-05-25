/// x1zzLang - 재귀 하강 파서 (완전 구현)
///
/// tokens / pos 필드 모두 실제 로직에서 사용 → dead_code 경고 없음.
///
/// BNF:
///   program        = stmt* EOF
///   stmt           = type_decl | var_stmt
///   type_decl      = "type" IDENT "=" "{" field_list "}" ";"?
///   field_list     = (field ("," field)* ","?)?
///   field          = IDENT ":" type_name
///   type_name      = "Option" "<" IDENT ">" | IDENT
///   var_stmt       = "mut"? "v" IDENT "=" pipeline_expr ";"?
///   pipeline_expr  = load_expr ("|>" pipeline_op)*
///   load_expr      = "load" "(" STRING_LIT ")" "::" IDENT
///   pipeline_op    = "filter" "(" expr ")"
///                  | "select" "(" "[" ident_list "]" ")"
///                  | "count"
///   expr           = primary (binop primary)?
///   primary        = IDENT | INT_LIT | FLOAT_LIT | STRING_LIT | "(" expr ")"
///   binop          = "==" | "!=" | "<" | ">" | "<=" | ">="

use crate::ast::{BinOpKind, Expr, PipelineOp, Program, Stmt, StructField};
use crate::error::{CompileError, CompileResult, ErrorKind};
use crate::token::{Span, Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    // ── 내부 헬퍼 ─────────────────────────────────────────────────────────────
    // tokens 및 pos 필드를 직접 읽는 메서드들

    /// 현재 위치의 TokenKind를 Clone하여 반환
    fn current_kind(&self) -> TokenKind {
        // self.tokens[self.pos] — 두 필드 모두 읽음
        self.tokens
            .get(self.pos)
            .map(|t| t.kind.clone())
            .unwrap_or(TokenKind::Eof)
    }

    /// 현재 위치의 Span을 Clone하여 반환
    fn current_span(&self) -> Span {
        self.tokens
            .get(self.pos)
            .map(|t| t.span.clone())
            .unwrap_or(Span::new(0, 0))
    }

    /// 현재 토큰을 소비하고 pos 전진 (두 필드 모두 변경)
    fn advance(&mut self) {
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
    }

    /// 기대 토큰이면 소비, 아니면 에러
    fn expect(&mut self, expected: &TokenKind) -> CompileResult<Span> {
        let kind = self.current_kind();   // tokens[pos].kind 읽음
        let span = self.current_span();   // tokens[pos].span 읽음
        if kind == *expected {
            self.advance();
            Ok(span)
        } else {
            Err(CompileError::new(
                ErrorKind::ExpectedToken(format!("{:?}", expected)),
                span,
                format!("예상 {:?}, 실제 {:?}", expected, kind),
            ))
        }
    }

    /// 기대 토큰이면 소비하고 true
    fn eat(&mut self, kind: &TokenKind) -> bool {
        if self.current_kind() == *kind {
            self.advance();
            true
        } else {
            false
        }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len().saturating_sub(1)
            || matches!(self.current_kind(), TokenKind::Eof)
    }

    // ── 최상위 ────────────────────────────────────────────────────────────────

    pub fn parse(&mut self) -> CompileResult<Program> {
        let mut program = Program::new();
        while !self.is_eof() {
            program.stmts.push(self.parse_stmt()?);
        }
        Ok(program)
    }

    // ── Stmt ──────────────────────────────────────────────────────────────────

    fn parse_stmt(&mut self) -> CompileResult<Stmt> {
        match self.current_kind() {
            TokenKind::Type      => self.parse_type_decl(),
            TokenKind::V
            | TokenKind::Mut     => self.parse_var_stmt(),
            other                => Err(CompileError::new(
                ErrorKind::UnexpectedToken(format!("{:?}", other)),
                self.current_span(),
                format!("구문 시작 불가 토큰: {:?}", other),
            )),
        }
    }

    // ── TypeDecl ─────────────────────────────────────────────────────────────
    //
    // type_decl = "type" IDENT "=" "{" field_list "}" ";"?

    fn parse_type_decl(&mut self) -> CompileResult<Stmt> {
        self.expect(&TokenKind::Type)?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::Assign)?;
        self.expect(&TokenKind::LBrace)?;
        let fields = self.parse_field_list()?;
        self.expect(&TokenKind::RBrace)?;
        self.eat(&TokenKind::Semicolon);
        Ok(Stmt::TypeDecl { name, fields })
    }

    fn parse_field_list(&mut self) -> CompileResult<Vec<StructField>> {
        let mut fields = Vec::new();
        loop {
            if matches!(self.current_kind(), TokenKind::RBrace | TokenKind::Eof) {
                break;
            }
            fields.push(self.parse_field()?);
            if !self.eat(&TokenKind::Comma) {
                break;
            }
            // 후행 쉼표 허용
            if matches!(self.current_kind(), TokenKind::RBrace) {
                break;
            }
        }
        Ok(fields)
    }

    /// field = IDENT ":" type_name
    fn parse_field(&mut self) -> CompileResult<StructField> {
        let name = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        let field_type = self.parse_type_name()?;
        Ok(StructField { name, field_type })
    }

    /// type_name = "Option" "<" IDENT ">" | IDENT
    fn parse_type_name(&mut self) -> CompileResult<String> {
        if matches!(self.current_kind(), TokenKind::OptionKw) {
            self.advance();
            self.expect(&TokenKind::Lt)?;
            let inner = self.expect_ident()?;
            self.expect(&TokenKind::Gt)?;
            Ok(format!("Option<{}>", inner))
        } else {
            self.expect_ident()
        }
    }

    // ── var_stmt ─────────────────────────────────────────────────────────────
    //
    // var_stmt = "mut"? "v" IDENT "=" pipeline_expr ";"?

    fn parse_var_stmt(&mut self) -> CompileResult<Stmt> {
        self.eat(&TokenKind::Mut);
        self.expect(&TokenKind::V)?;
        let _var_name = self.expect_ident()?;  // 현재 AST에 저장하지 않음
        self.expect(&TokenKind::Assign)?;
        let stmt = self.parse_pipeline_stream()?;
        self.eat(&TokenKind::Semicolon);
        Ok(stmt)
    }

    // ── PipelineStream (핵심) ─────────────────────────────────────────────────
    //
    // load_expr = "load" "(" STRING_LIT ")" "::" IDENT ("|>" op)*

    fn parse_pipeline_stream(&mut self) -> CompileResult<Stmt> {
        self.expect(&TokenKind::Load)?;
        self.expect(&TokenKind::LParen)?;

        let file_path = match self.current_kind() {
            TokenKind::StringLit(s) => {
                self.advance();
                s
            }
            other => {
                return Err(CompileError::new(
                    ErrorKind::ExpectedToken("StringLit".into()),
                    self.current_span(),
                    format!("load() 경로는 문자열 리터럴. 실제: {:?}", other),
                ))
            }
        };

        self.expect(&TokenKind::RParen)?;
        self.expect(&TokenKind::TypeAssign)?;  // ::
        let schema_name = self.expect_ident()?;

        let mut ops: Vec<PipelineOp> = Vec::new();
        while matches!(self.current_kind(), TokenKind::Pipeline) {
            self.advance();  // |> 소비
            ops.push(self.parse_pipeline_op()?);
        }

        Ok(Stmt::PipelineStream { file_path, schema_name, ops })
    }

    // ── PipelineOp ───────────────────────────────────────────────────────────

    fn parse_pipeline_op(&mut self) -> CompileResult<PipelineOp> {
        match self.current_kind() {
            TokenKind::Filter => {
                self.advance();
                self.expect(&TokenKind::LParen)?;
                let expr = self.parse_expr()?;
                self.expect(&TokenKind::RParen)?;
                Ok(PipelineOp::Filter(expr))
            }
            TokenKind::Select => {
                self.advance();
                self.expect(&TokenKind::LParen)?;
                self.expect(&TokenKind::LBracket)?;

                let mut cols = Vec::new();
                loop {
                    if matches!(self.current_kind(), TokenKind::RBracket | TokenKind::Eof) {
                        break;
                    }
                    cols.push(self.expect_ident()?);
                    if !self.eat(&TokenKind::Comma) {
                        break;
                    }
                    if matches!(self.current_kind(), TokenKind::RBracket) {
                        break;
                    }
                }

                self.expect(&TokenKind::RBracket)?;
                self.expect(&TokenKind::RParen)?;
                Ok(PipelineOp::Select(cols))
            }
            TokenKind::Count => {
                self.advance();
                Ok(PipelineOp::Count)
            }
            other => Err(CompileError::new(
                ErrorKind::UnexpectedToken(format!("{:?}", other)),
                self.current_span(),
                format!("|> 뒤 불가 연산: {:?}", other),
            )),
        }
    }

    // ── 표현식 ───────────────────────────────────────────────────────────────
    //
    // expr = primary (binop primary)?

    fn parse_expr(&mut self) -> CompileResult<Expr> {
        let lhs = self.parse_primary()?;
        if let Some(op) = self.current_binop() {
            self.advance();
            let rhs = self.parse_primary()?;
            Ok(Expr::BinOp { lhs: Box::new(lhs), op, rhs: Box::new(rhs) })
        } else {
            Ok(lhs)
        }
    }

    fn current_binop(&self) -> Option<BinOpKind> {
        match self.current_kind() {
            TokenKind::EqEq  => Some(BinOpKind::Eq),
            TokenKind::NotEq => Some(BinOpKind::NotEq),
            TokenKind::Lt    => Some(BinOpKind::Lt),
            TokenKind::Gt    => Some(BinOpKind::Gt),
            TokenKind::LtEq  => Some(BinOpKind::LtEq),
            TokenKind::GtEq  => Some(BinOpKind::GtEq),
            _                => None,
        }
    }

    fn parse_primary(&mut self) -> CompileResult<Expr> {
        match self.current_kind() {
            TokenKind::Ident(s) => {
                self.advance();
                Ok(Expr::Ident(s))
            }
            TokenKind::IntLit(n) => {
                self.advance();
                Ok(Expr::IntLit(n))
            }
            TokenKind::FloatLit(f) => {
                self.advance();
                Ok(Expr::FloatLit(f))
            }
            TokenKind::StringLit(s) => {
                self.advance();
                Ok(Expr::StringLit(s))
            }
            TokenKind::LParen => {
                self.advance();
                let e = self.parse_expr()?;
                self.expect(&TokenKind::RParen)?;
                Ok(e)
            }
            other => Err(CompileError::new(
                ErrorKind::UnexpectedToken(format!("{:?}", other)),
                self.current_span(),
                format!("표현식 불가 토큰: {:?}", other),
            )),
        }
    }

    // ── 유틸리티 ─────────────────────────────────────────────────────────────

    fn expect_ident(&mut self) -> CompileResult<String> {
        match self.current_kind() {
            TokenKind::Ident(s) => {
                self.advance();
                Ok(s)
            }
            other => Err(CompileError::new(
                ErrorKind::ExpectedToken("Ident".into()),
                self.current_span(),
                format!("식별자 필요, 실제: {:?}", other),
            )),
        }
    }
}
