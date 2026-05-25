/// x1zzLang - 재귀 하강 파서 (완전 구현)
///
/// 지원 문법:
///
///   program        = stmt* EOF
///   stmt           = type_decl | var_stmt
///
///   type_decl      = "type" IDENT "=" "{" field_list "}" ";"
///   field_list     = (field ("," field)* ","?)?
///   field          = IDENT ":" type_name
///   type_name      = "Option" "<" IDENT ">" | IDENT
///
///   var_stmt       = ("mut")? "v" IDENT "=" pipeline_expr ";"
///   pipeline_expr  = load_expr ("|>" pipeline_op)*
///   load_expr      = "load" "(" STRING_LIT ")" "::" IDENT
///
///   pipeline_op    = filter_op | select_op | "count"
///   filter_op      = "filter" "(" expr ")"
///   select_op      = "select" "(" "[" ident_list "]" ")"
///   ident_list     = (IDENT ("," IDENT)* ","?)?
///
///   expr           = primary (binop primary)?
///   primary        = IDENT | INT_LIT | FLOAT_LIT | STRING_LIT | "(" expr ")"
///   binop          = ">" | "<" | ">=" | "<=" | "==" | "!="

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

    // ── 기본 헬퍼 ────────────────────────────────────────────────────────────

    /// 현재 토큰 kind를 Clone하여 반환 (빌림 검사기 안전)
    fn current_kind(&self) -> TokenKind {
        self.tokens
            .get(self.pos)
            .map(|t| t.kind.clone())
            .unwrap_or(TokenKind::Eof)
    }

    /// 현재 토큰 스팬을 Clone하여 반환
    fn current_span(&self) -> Span {
        self.tokens
            .get(self.pos)
            .map(|t| t.span.clone())
            .unwrap_or(Span::new(0, 0))
    }

    /// 현재 토큰을 소비하고 pos를 전진 (반환값 없음 — 빌림 검사기 안전)
    fn advance(&mut self) {
        if self.pos < self.tokens.len().saturating_sub(1) {
            self.pos += 1;
        }
    }

    /// 현재 kind가 expected와 같으면 소비, 아니면 에러
    fn expect(&mut self, expected: &TokenKind) -> CompileResult<Span> {
        let kind = self.current_kind();
        let span = self.current_span();
        if kind == *expected {
            self.advance();
            Ok(span)
        } else {
            Err(CompileError::new(
                ErrorKind::ExpectedToken(format!("{:?}", expected)),
                span,
                format!("예상: {:?}, 실제: {:?}", expected, kind),
            ))
        }
    }

    /// 현재 kind가 일치하면 소비하고 true 반환
    fn eat(&mut self, kind: &TokenKind) -> bool {
        if self.current_kind() == *kind {
            self.advance();
            true
        } else {
            false
        }
    }

    fn is_eof(&self) -> bool {
        matches!(self.current_kind(), TokenKind::Eof)
    }

    // ── 최상위 파서 ──────────────────────────────────────────────────────────

    pub fn parse(&mut self) -> CompileResult<Program> {
        let mut program = Program::new();
        while !self.is_eof() {
            let stmt = self.parse_stmt()?;
            program.stmts.push(stmt);
        }
        Ok(program)
    }

    // ── 구문(Stmt) 파서 ───────────────────────────────────────────────────────

    fn parse_stmt(&mut self) -> CompileResult<Stmt> {
        // current_kind() 를 먼저 clone하여 self 의 빌림을 해제한 뒤 분기
        match self.current_kind() {
            TokenKind::Type       => self.parse_type_decl(),
            TokenKind::V
            | TokenKind::Mut      => self.parse_var_stmt(),
            other                 => Err(CompileError::new(
                ErrorKind::UnexpectedToken(format!("{:?}", other)),
                self.current_span(),
                format!("구문 시작에 올 수 없는 토큰: {:?}", other),
            )),
        }
    }

    // ── TypeDecl 파서 ─────────────────────────────────────────────────────────
    //
    // type_decl = "type" IDENT "=" "{" field_list "}" ";"

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
            match self.current_kind() {
                TokenKind::RBrace | TokenKind::Eof => break,
                _ => {}
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

    // ── var_stmt 파서 ─────────────────────────────────────────────────────────
    //
    // var_stmt = ("mut")? "v" IDENT "=" pipeline_expr ";"

    fn parse_var_stmt(&mut self) -> CompileResult<Stmt> {
        self.eat(&TokenKind::Mut);             // mut 선택적 소비 (AST에 저장하지 않음)
        self.expect(&TokenKind::V)?;
        let _var_name = self.expect_ident()?;  // 변수명 (현재 AST 구조에 저장하지 않음)
        self.expect(&TokenKind::Assign)?;
        let stmt = self.parse_pipeline_expr()?;
        self.eat(&TokenKind::Semicolon);
        Ok(stmt)
    }

    // ── pipeline_expr 파서 ────────────────────────────────────────────────────
    //
    // pipeline_expr = load_expr ("|>" pipeline_op)*
    // load_expr     = "load" "(" STRING_LIT ")" "::" IDENT

    fn parse_pipeline_expr(&mut self) -> CompileResult<Stmt> {
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
                    format!("load() 에 문자열 리터럴 필요. 실제: {:?}", other),
                ));
            }
        };

        self.expect(&TokenKind::RParen)?;
        self.expect(&TokenKind::TypeAssign)?; // ::
        let schema_name = self.expect_ident()?;

        let mut ops: Vec<PipelineOp> = Vec::new();
        while matches!(self.current_kind(), TokenKind::Pipeline) {
            self.advance(); // |> 소비
            let op = self.parse_pipeline_op()?;
            ops.push(op);
        }

        Ok(Stmt::PipelineStream {
            file_path,
            schema_name,
            ops,
        })
    }

    // ── pipeline_op 파서 ──────────────────────────────────────────────────────

    fn parse_pipeline_op(&mut self) -> CompileResult<PipelineOp> {
        match self.current_kind() {
            TokenKind::Filter => self.parse_filter_op(),
            TokenKind::Select => self.parse_select_op(),
            TokenKind::Count  => {
                self.advance();
                Ok(PipelineOp::Count)
            }
            other => Err(CompileError::new(
                ErrorKind::UnexpectedToken(format!("{:?}", other)),
                self.current_span(),
                format!("|> 뒤에 올 수 없는 연산: {:?}", other),
            )),
        }
    }

    /// filter_op = "filter" "(" expr ")"
    fn parse_filter_op(&mut self) -> CompileResult<PipelineOp> {
        self.expect(&TokenKind::Filter)?;
        self.expect(&TokenKind::LParen)?;
        let expr = self.parse_expr()?;
        self.expect(&TokenKind::RParen)?;
        Ok(PipelineOp::Filter(expr))
    }

    /// select_op = "select" "(" "[" ident_list "]" ")"
    fn parse_select_op(&mut self) -> CompileResult<PipelineOp> {
        self.expect(&TokenKind::Select)?;
        self.expect(&TokenKind::LParen)?;
        self.expect(&TokenKind::LBracket)?;

        let mut cols: Vec<String> = Vec::new();
        loop {
            match self.current_kind() {
                TokenKind::RBracket | TokenKind::Eof => break,
                _ => {}
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

    // ── 표현식 파서 ───────────────────────────────────────────────────────────
    //
    // expr    = primary (binop primary)?
    // primary = IDENT | INT_LIT | FLOAT_LIT | STRING_LIT | "(" expr ")"
    // binop   = ">" | "<" | ">=" | "<=" | "==" | "!="

    fn parse_expr(&mut self) -> CompileResult<Expr> {
        let lhs = self.parse_primary()?;
        if let Some(op) = self.current_binop() {
            self.advance(); // 연산자 소비
            let rhs = self.parse_primary()?;
            Ok(Expr::BinOp {
                lhs: Box::new(lhs),
                op,
                rhs: Box::new(rhs),
            })
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
                format!("표현식에 올 수 없는 토큰: {:?}", other),
            )),
        }
    }

    // ── 유틸리티 ─────────────────────────────────────────────────────────────

    /// 현재 토큰이 Ident이면 소비하고 String 반환
    fn expect_ident(&mut self) -> CompileResult<String> {
        match self.current_kind() {
            TokenKind::Ident(s) => {
                self.advance();
                Ok(s)
            }
            other => Err(CompileError::new(
                ErrorKind::ExpectedToken("Ident".into()),
                self.current_span(),
                format!("식별자가 필요합니다. 실제: {:?}", other),
            )),
        }
    }
}
