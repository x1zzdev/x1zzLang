/// x1zzLang - 재귀 하강 파서 (v0.16 완전 구현)
///
/// BNF:
///   program        = stmt* EOF
///   stmt           = type_decl | var_stmt
///   type_decl      = "type" IDENT "=" "{" field_list "}" ";"?
///   field_list     = (field ("," field)* ","?)?
///   field          = IDENT ":" type_name
///   type_name      = "Option" "<" IDENT ">" | IDENT
///   var_stmt       = "mut"? "v" IDENT "=" pipeline_expr ";"?
///   pipeline_expr  = (load_expr | var_ref_expr) ("|>" pipeline_op)*
///   load_expr      = "load" "(" STRING_LIT ")" "::" IDENT
///   var_ref_expr   = IDENT  (기존 변수 참조)
///   pipeline_op    = "filter" "(" expr ")"
///                  | "select" "(" "[" ident_list "]" ")"
///                  | "count" ("(" STRING_LIT ")")?
///                  | "groupBy" "(" STRING_LIT ")"
///                  | "sum"     "(" STRING_LIT ")"
///                  | "mean"    "(" STRING_LIT ")"
///                  | "min"     "(" STRING_LIT ")"
///                  | "max"     "(" STRING_LIT ")"
///                  | "orderBy" "(" STRING_LIT ("," "desc" ":" BOOL)? ")"
///                  | "take"    "(" INT_LIT ")"
///                  | "dropNull" "(" STRING_LIT ")"
///                  | "fillNull" "(" STRING_LIT "," (INT_LIT | FLOAT_LIT | STRING_LIT) ")"
///   expr           = primary (binop primary)?
///   primary        = "col" "(" STRING_LIT ")"   ← col("x") → Ident(x)
///                  | IDENT | INT_LIT | FLOAT_LIT | STRING_LIT
///                  | "true" | "false"
///                  | "(" expr ")"
///   binop          = "==" | "!=" | "<" | ">" | "<=" | ">="
///
/// [v0.16 변경사항]
///   - col("col_name") 표현식 지원: col("x") → Expr::Ident("x")
///   - true / false 불리언 리터럴 지원
///   - Count(None) / Count(Some(col)) 구분
///   - 9종 신규 파이프라인 연산자 파싱

use crate::ast::{
    BinOpKind, Expr, FillNullValue, PipelineOp, PipelineSource, Program, Stmt, StructField,
};
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

    fn current_kind(&self) -> TokenKind {
        self.tokens
            .get(self.pos)
            .map(|t| t.kind.clone())
            .unwrap_or(TokenKind::Eof)
    }

    fn current_span(&self) -> Span {
        self.tokens
            .get(self.pos)
            .map(|t| t.span.clone())
            .unwrap_or(Span::new(0, 0))
    }

    fn advance(&mut self) {
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
    }

    fn expect(&mut self, expected: &TokenKind) -> CompileResult<Span> {
        let kind = self.current_kind();
        let span = self.current_span();
        if kind == *expected {
            self.advance();
            Ok(span)
        } else {
            Err(CompileError::new(
                ErrorKind::ExpectedToken(format!("{:?}", expected)),
                span.clone(),
                format!(
                    "예상 토큰 {:?} 없음, 실제: {:?}",
                    expected, kind
                ),
            ))
        }
    }

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
            TokenKind::Type       => self.parse_type_decl(),
            TokenKind::V
            | TokenKind::Mut     => self.parse_var_stmt(),
            other                => Err(CompileError::new(
                ErrorKind::UnexpectedToken(format!("{:?}", other)),
                self.current_span(),
                format!("구문 시작 불가 토큰: {:?}", other),
            )),
        }
    }

    // ── TypeDecl ──────────────────────────────────────────────────────────────
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
            if matches!(self.current_kind(), TokenKind::RBrace) {
                break;
            }
        }
        Ok(fields)
    }

    fn parse_field(&mut self) -> CompileResult<StructField> {
        let name = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        let field_type = self.parse_type_name()?;
        Ok(StructField { name, field_type })
    }

    fn parse_type_name(&mut self) -> CompileResult<String> {
        if matches!(self.current_kind(), TokenKind::OptionKw) {
            let span = self.current_span();
            self.advance();
            if !matches!(self.current_kind(), TokenKind::Lt) {
                return Err(CompileError::new(
                    ErrorKind::ExpectedToken("<".into()),
                    span,
                    "Option 뒤에는 '<' 가 와야 합니다. 예: Option<float>",
                ));
            }
            self.expect(&TokenKind::Lt)?;
            let inner = self.expect_ident()?;
            self.expect(&TokenKind::Gt)?;
            Ok(format!("Option<{}>", inner))
        } else {
            self.expect_ident()
        }
    }

    // ── var_stmt ──────────────────────────────────────────────────────────────
    // var_stmt = "mut"? "v" IDENT "=" pipeline_expr ";"?

    fn parse_var_stmt(&mut self) -> CompileResult<Stmt> {
        let is_mut = self.eat(&TokenKind::Mut);
        self.expect(&TokenKind::V)?;
        let var_name = self.expect_ident()?;
        self.expect(&TokenKind::Assign)?;
        let (source, ops) = self.parse_pipeline_expr()?;
        self.eat(&TokenKind::Semicolon);
        Ok(Stmt::VarDecl { var_name, is_mut, source, ops })
    }

    // ── 파이프라인 표현식 ──────────────────────────────────────────────────────
    // pipeline_expr = (load_expr | var_ref_expr) ("|>" pipeline_op)*

    fn parse_pipeline_expr(&mut self) -> CompileResult<(PipelineSource, Vec<PipelineOp>)> {
        // 소스 결정: load(...) 또는 변수 참조
        let source = match self.current_kind() {
            TokenKind::Load => self.parse_load_source()?,
            TokenKind::Ident(name) => {
                let var_name = name.clone();
                self.advance();
                PipelineSource::VarRef(var_name)
            }
            other => {
                return Err(CompileError::new(
                    ErrorKind::UnexpectedToken(format!("{:?}", other)),
                    self.current_span(),
                    format!(
                        "파이프라인은 load(...) 또는 변수 참조로 시작해야 합니다. 실제: {:?}",
                        other
                    ),
                ));
            }
        };

        // |> 연산자 체이닝
        let mut ops: Vec<PipelineOp> = Vec::new();
        while matches!(self.current_kind(), TokenKind::Pipeline) {
            self.advance(); // |> 소비
            ops.push(self.parse_pipeline_op()?);
        }

        Ok((source, ops))
    }

    // load_expr = "load" "(" STRING_LIT ")" "::" IDENT
    fn parse_load_source(&mut self) -> CompileResult<PipelineSource> {
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
                    format!("load() 경로는 문자열 리터럴이어야 합니다. 실제: {:?}", other),
                ));
            }
        };

        self.expect(&TokenKind::RParen)?;

        // :: 뒤에 스키마 이름
        if !matches!(self.current_kind(), TokenKind::TypeAssign) {
            let span = self.current_span();
            return Err(CompileError::new(
                ErrorKind::ExpectedToken("::".into()),
                span,
                "'::' 뒤에는 스키마 이름이 와야 합니다. 예: load(\"data.csv\") :: AirQuality",
            ));
        }
        self.expect(&TokenKind::TypeAssign)?; // ::
        let schema_name = self.expect_ident()?;

        Ok(PipelineSource::Load { file_path, schema_name })
    }

    // ── PipelineOp ────────────────────────────────────────────────────────────

    fn parse_pipeline_op(&mut self) -> CompileResult<PipelineOp> {
        match self.current_kind() {
            // ── 기존 연산자 ──────────────────────────────────────────────────
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
                // count("col") 형식이면 컬럼 집계 / count (인수 없음) 이면 전체 행 수
                if matches!(self.current_kind(), TokenKind::LParen) {
                    self.advance();
                    let col_name = self.expect_string_lit()?;
                    self.expect(&TokenKind::RParen)?;
                    Ok(PipelineOp::Count(Some(col_name)))
                } else {
                    Ok(PipelineOp::Count(None))
                }
            }

            // ── v0.16 신규 집계 연산자 ────────────────────────────────────────
            TokenKind::GroupBy => {
                self.advance();
                self.expect(&TokenKind::LParen)?;
                let col_name = self.expect_string_lit()?;
                self.expect(&TokenKind::RParen)?;
                Ok(PipelineOp::GroupBy(col_name))
            }
            TokenKind::Sum => {
                self.advance();
                self.expect(&TokenKind::LParen)?;
                let col_name = self.expect_string_lit()?;
                self.expect(&TokenKind::RParen)?;
                Ok(PipelineOp::Sum(col_name))
            }
            TokenKind::Mean => {
                self.advance();
                self.expect(&TokenKind::LParen)?;
                let col_name = self.expect_string_lit()?;
                self.expect(&TokenKind::RParen)?;
                Ok(PipelineOp::Mean(col_name))
            }
            TokenKind::Min => {
                self.advance();
                self.expect(&TokenKind::LParen)?;
                let col_name = self.expect_string_lit()?;
                self.expect(&TokenKind::RParen)?;
                Ok(PipelineOp::Min(col_name))
            }
            TokenKind::Max => {
                self.advance();
                self.expect(&TokenKind::LParen)?;
                let col_name = self.expect_string_lit()?;
                self.expect(&TokenKind::RParen)?;
                Ok(PipelineOp::Max(col_name))
            }

            // ── v0.16 정렬 / 슬라이싱 ─────────────────────────────────────────
            TokenKind::OrderBy => {
                self.advance();
                self.expect(&TokenKind::LParen)?;
                let col_name = self.expect_string_lit()?;
                // 선택적 desc: true/false
                let desc = if matches!(self.current_kind(), TokenKind::Comma) {
                    self.advance(); // , 소비
                    self.expect(&TokenKind::Desc)?;
                    self.expect(&TokenKind::Colon)?;
                    match self.current_kind() {
                        TokenKind::True  => { self.advance(); true }
                        TokenKind::False => { self.advance(); false }
                        other => {
                            return Err(CompileError::new(
                                ErrorKind::ExpectedToken("true or false".into()),
                                self.current_span(),
                                format!(
                                    "orderBy 의 desc: 뒤에는 true 또는 false 가 와야 합니다. 실제: {:?}",
                                    other
                                ),
                            ));
                        }
                    }
                } else {
                    false // 기본값: 오름차순
                };
                self.expect(&TokenKind::RParen)?;
                Ok(PipelineOp::OrderBy { col: col_name, desc })
            }
            TokenKind::Take => {
                self.advance();
                self.expect(&TokenKind::LParen)?;
                let n = match self.current_kind() {
                    TokenKind::IntLit(n) => { self.advance(); n }
                    other => {
                        return Err(CompileError::new(
                            ErrorKind::ExpectedToken("IntLit".into()),
                            self.current_span(),
                            format!("take() 에는 정수 리터럴이 필요합니다. 실제: {:?}", other),
                        ));
                    }
                };
                self.expect(&TokenKind::RParen)?;
                Ok(PipelineOp::Take(n))
            }

            // ── v0.16 Null 처리 ────────────────────────────────────────────────
            TokenKind::DropNull => {
                self.advance();
                self.expect(&TokenKind::LParen)?;
                let col_name = self.expect_string_lit()?;
                self.expect(&TokenKind::RParen)?;
                Ok(PipelineOp::DropNull(col_name))
            }
            TokenKind::FillNull => {
                self.advance();
                self.expect(&TokenKind::LParen)?;
                let col_name = self.expect_string_lit()?;
                self.expect(&TokenKind::Comma)?;
                // 채우기 값: 정수, 부동소수 또는 문자열 리터럴
                let value = match self.current_kind() {
                    TokenKind::IntLit(n)   => { self.advance(); FillNullValue::Int(n) }
                    TokenKind::FloatLit(f) => { self.advance(); FillNullValue::Float(f) }
                    TokenKind::StringLit(s) => { self.advance(); FillNullValue::Str(s) }
                    other => {
                        return Err(CompileError::new(
                            ErrorKind::ExpectedToken("number or string".into()),
                            self.current_span(),
                            format!(
                                "fillNull() 채우기 값은 정수, 부동소수 또는 문자열이어야 합니다. 실제: {:?}",
                                other
                            ),
                        ));
                    }
                };
                self.expect(&TokenKind::RParen)?;
                Ok(PipelineOp::FillNull { col: col_name, value })
            }

            other => Err(CompileError::new(
                ErrorKind::UnexpectedToken(format!("{:?}", other)),
                self.current_span(),
                format!(
                    "|> 뒤에는 filter/select/count/groupBy/sum/mean/min/max/orderBy/take/dropNull/fillNull 중 하나가 와야 합니다. 실제: {:?}",
                    other
                ),
            )),
        }
    }

    // ── 표현식 ────────────────────────────────────────────────────────────────
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
                // col("column_name") 함수 호출 형태 처리
                // col("x") → Expr::Ident("x")  (런타임에서 polars::col("x") 로 변환됨)
                if s == "col" && matches!(self.current_kind(), TokenKind::LParen) {
                    self.advance(); // ( 소비
                    let col_name = match self.current_kind() {
                        TokenKind::StringLit(name) => { self.advance(); name }
                        other => {
                            return Err(CompileError::new(
                                ErrorKind::ExpectedToken("StringLit".into()),
                                self.current_span(),
                                format!(
                                    "col() 안에는 문자열 리터럴이 필요합니다. 예: col(\"income\"). 실제: {:?}",
                                    other
                                ),
                            ));
                        }
                    };
                    self.expect(&TokenKind::RParen)?;
                    Ok(Expr::Ident(col_name))
                } else {
                    Ok(Expr::Ident(s))
                }
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
            // 불리언 리터럴 (v0.16)
            TokenKind::True => {
                self.advance();
                Ok(Expr::BoolLit(true))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr::BoolLit(false))
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
                format!("표현식에 사용할 수 없는 토큰: {:?}", other),
            )),
        }
    }

    // ── 유틸리티 ──────────────────────────────────────────────────────────────

    fn expect_ident(&mut self) -> CompileResult<String> {
        match self.current_kind() {
            TokenKind::Ident(s) => {
                self.advance();
                Ok(s)
            }
            other => Err(CompileError::new(
                ErrorKind::ExpectedToken("Ident".into()),
                self.current_span(),
                format!("식별자(변수명/컬럼명)가 필요합니다. 실제: {:?}", other),
            )),
        }
    }

    /// 현재 토큰이 StringLit이면 소비하고 반환, 아니면 에러
    fn expect_string_lit(&mut self) -> CompileResult<String> {
        match self.current_kind() {
            TokenKind::StringLit(s) => {
                self.advance();
                Ok(s)
            }
            other => Err(CompileError::new(
                ErrorKind::ExpectedToken("StringLit".into()),
                self.current_span(),
                format!("문자열 리터럴이 필요합니다. 실제: {:?}", other),
            )),
        }
    }
}

// ── 유닛 테스트 ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::ast::{Expr, BinOpKind, FillNullValue, PipelineOp, PipelineSource, Stmt, StructField};

    fn parse_src(src: &str) -> CompileResult<Program> {
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    // ── 테스트 1: 변수 선언 파싱 및 VarDecl AST 빌드 ────────────────────────
    #[test]
    fn test_var_decl_and_pipeline_op() {
        let src = r#"v result = load("data.csv") :: AirQuality |> count;"#;
        let program = parse_src(src).expect("파싱 실패");
        assert_eq!(program.stmts.len(), 1);

        match &program.stmts[0] {
            Stmt::VarDecl { var_name, is_mut, source, ops } => {
                assert_eq!(var_name, "result");
                assert!(!is_mut);
                assert_eq!(
                    source,
                    &PipelineSource::Load {
                        file_path: "data.csv".into(),
                        schema_name: "AirQuality".into(),
                    }
                );
                assert_eq!(ops.len(), 1);
                assert_eq!(ops[0], PipelineOp::Count(None));
            }
            other => panic!("VarDecl 예상, 실제: {:?}", other),
        }
    }

    // ── 테스트 2: load :: filter 파싱 및 AST BinOp 검증 ─────────────────────
    #[test]
    fn test_load_filter_select_ast() {
        let src =
            r#"v air = load("seoul.csv") :: AirQuality |> filter(pm10 > 50) |> select([station, date]);"#;
        let program = parse_src(src).expect("파싱 실패");
        assert_eq!(program.stmts.len(), 1);

        match &program.stmts[0] {
            Stmt::VarDecl { var_name, source, ops, .. } => {
                assert_eq!(var_name, "air");
                assert_eq!(
                    source,
                    &PipelineSource::Load {
                        file_path: "seoul.csv".into(),
                        schema_name: "AirQuality".into(),
                    }
                );
                assert_eq!(ops.len(), 2);

                // filter(pm10 > 50)
                match &ops[0] {
                    PipelineOp::Filter(expr) => match expr {
                        Expr::BinOp { lhs, op, rhs } => {
                            assert_eq!(**lhs, Expr::Ident("pm10".into()));
                            assert_eq!(*op, BinOpKind::Gt);
                            assert_eq!(**rhs, Expr::IntLit(50));
                        }
                        _ => panic!("BinOp 예상"),
                    },
                    _ => panic!("Filter 예상"),
                }

                // select([station, date])
                match &ops[1] {
                    PipelineOp::Select(cols) => {
                        assert_eq!(cols, &vec!["station".to_string(), "date".to_string()]);
                    }
                    _ => panic!("Select 예상"),
                }
            }
            other => panic!("VarDecl 예상, 실제: {:?}", other),
        }
    }

    // ── 테스트 3: TypeDecl 파싱 검증 ────────────────────────────────────────
    #[test]
    fn test_type_decl_parsing() {
        let src = r#"
type AirQuality = {
  station: string,
  pm10: Option<float>,
};
"#;
        let program = parse_src(src).expect("파싱 실패");
        assert_eq!(program.stmts.len(), 1);
        match &program.stmts[0] {
            Stmt::TypeDecl { name, fields } => {
                assert_eq!(name, "AirQuality");
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0], StructField { name: "station".into(), field_type: "string".into() });
                assert_eq!(fields[1], StructField { name: "pm10".into(), field_type: "Option<float>".into() });
            }
            other => panic!("TypeDecl 예상, 실제: {:?}", other),
        }
    }

    // ── 테스트 4: VarRef (변수 참조) 파싱 ────────────────────────────────────
    #[test]
    fn test_var_ref_pipeline() {
        let src = r#"v filtered = air |> filter(pm25 > 10);"#;
        let program = parse_src(src).expect("파싱 실패");
        assert_eq!(program.stmts.len(), 1);
        match &program.stmts[0] {
            Stmt::VarDecl { var_name, source, ops, .. } => {
                assert_eq!(var_name, "filtered");
                assert_eq!(source, &PipelineSource::VarRef("air".into()));
                assert_eq!(ops.len(), 1);
            }
            other => panic!("VarDecl 예상, 실제: {:?}", other),
        }
    }

    // ── 테스트 5: :: 없이 load 시 에러 ───────────────────────────────────────
    #[test]
    fn test_missing_schema_error() {
        let src = r#"v x = load("data.csv") |> count;"#;
        let result = parse_src(src);
        assert!(
            result.is_err(),
            "스키마 없이 load 하면 에러여야 함"
        );
        let err = result.unwrap_err();
        assert!(
            err.message.contains("::") || format!("{}", err).contains("::"),
            "에러 메시지에 '::' 포함돼야 함: {}",
            err
        );
    }

    // ── 테스트 6: mut 변수 선언 ───────────────────────────────────────────────
    #[test]
    fn test_mut_var_decl() {
        let src = r#"mut v data = load("file.csv") :: Schema;"#;
        let program = parse_src(src).expect("파싱 실패");
        match &program.stmts[0] {
            Stmt::VarDecl { is_mut, .. } => assert!(*is_mut),
            other => panic!("VarDecl 예상: {:?}", other),
        }
    }

    // ── 테스트 7 (v0.16): col("x") 표현식 파싱 ──────────────────────────────
    #[test]
    fn test_col_function_in_filter() {
        let src = r#"v result = data |> filter(col("income") < 1_200_000);"#;
        let program = parse_src(src).expect("파싱 실패");
        match &program.stmts[0] {
            Stmt::VarDecl { ops, .. } => {
                assert_eq!(ops.len(), 1);
                match &ops[0] {
                    PipelineOp::Filter(Expr::BinOp { lhs, op, rhs }) => {
                        assert_eq!(**lhs, Expr::Ident("income".into()), "col(\"income\") → Ident(income) 실패");
                        assert_eq!(*op, BinOpKind::Lt);
                        assert_eq!(**rhs, Expr::IntLit(1_200_000));
                    }
                    other => panic!("Filter(BinOp) 예상: {:?}", other),
                }
            }
            other => panic!("VarDecl 예상: {:?}", other),
        }
    }

    // ── 테스트 8 (v0.16): 불리언 리터럴 파싱 ────────────────────────────────
    #[test]
    fn test_boolean_literal_in_filter() {
        let src = r#"v result = data |> filter(col("support") == false);"#;
        let program = parse_src(src).expect("파싱 실패");
        match &program.stmts[0] {
            Stmt::VarDecl { ops, .. } => {
                match &ops[0] {
                    PipelineOp::Filter(Expr::BinOp { lhs, op, rhs }) => {
                        assert_eq!(**lhs, Expr::Ident("support".into()));
                        assert_eq!(*op, BinOpKind::Eq);
                        assert_eq!(**rhs, Expr::BoolLit(false));
                    }
                    other => panic!("Filter(BinOp) 예상: {:?}", other),
                }
            }
            other => panic!("VarDecl 예상: {:?}", other),
        }
    }

    // ── 테스트 9 (v0.16): groupBy + count 파싱 ───────────────────────────────
    #[test]
    fn test_group_by_and_count() {
        let src = r#"v result = data |> groupBy("region") |> count("population");"#;
        let program = parse_src(src).expect("파싱 실패");
        match &program.stmts[0] {
            Stmt::VarDecl { ops, .. } => {
                assert_eq!(ops.len(), 2);
                assert_eq!(ops[0], PipelineOp::GroupBy("region".into()));
                assert_eq!(ops[1], PipelineOp::Count(Some("population".into())));
            }
            other => panic!("VarDecl 예상: {:?}", other),
        }
    }

    // ── 테스트 10 (v0.16): mean + orderBy(desc) + take 파싱 ─────────────────
    #[test]
    fn test_mean_orderby_take() {
        let src = r#"v result = data |> groupBy("region") |> mean("income") |> orderBy("income", desc: true) |> take(10);"#;
        let program = parse_src(src).expect("파싱 실패");
        match &program.stmts[0] {
            Stmt::VarDecl { ops, .. } => {
                assert_eq!(ops.len(), 4);
                assert_eq!(ops[0], PipelineOp::GroupBy("region".into()));
                assert_eq!(ops[1], PipelineOp::Mean("income".into()));
                assert_eq!(ops[2], PipelineOp::OrderBy { col: "income".into(), desc: true });
                assert_eq!(ops[3], PipelineOp::Take(10));
            }
            other => panic!("VarDecl 예상: {:?}", other),
        }
    }

    // ── 테스트 11 (v0.16): dropNull + fillNull 파싱 ──────────────────────────
    #[test]
    fn test_drop_null_and_fill_null() {
        let src = r#"v result = data |> dropNull("income") |> fillNull("income", 0);"#;
        let program = parse_src(src).expect("파싱 실패");
        match &program.stmts[0] {
            Stmt::VarDecl { ops, .. } => {
                assert_eq!(ops.len(), 2);
                assert_eq!(ops[0], PipelineOp::DropNull("income".into()));
                assert_eq!(
                    ops[1],
                    PipelineOp::FillNull {
                        col: "income".into(),
                        value: FillNullValue::Int(0),
                    }
                );
            }
            other => panic!("VarDecl 예상: {:?}", other),
        }
    }

    // ── 테스트 12 (v0.16): sum / min / max 파싱 ─────────────────────────────
    #[test]
    fn test_sum_min_max_parse() {
        let src = r#"v result = data |> groupBy("region") |> sum("pop") |> min("price") |> max("price");"#;
        let program = parse_src(src).expect("파싱 실패");
        match &program.stmts[0] {
            Stmt::VarDecl { ops, .. } => {
                assert_eq!(ops.len(), 4);
                assert_eq!(ops[0], PipelineOp::GroupBy("region".into()));
                assert_eq!(ops[1], PipelineOp::Sum("pop".into()));
                assert_eq!(ops[2], PipelineOp::Min("price".into()));
                assert_eq!(ops[3], PipelineOp::Max("price".into()));
            }
            other => panic!("VarDecl 예상: {:?}", other),
        }
    }

    // ── 테스트 13 (v0.16): README 전체 파이프라인 파싱 ──────────────────────
    #[test]
    fn test_readme_full_pipeline() {
        let src = r#"v blind_spots = data
  |> dropNull("income")
  |> filter(col("income") < 1_200_000)
  |> filter(col("support") == false)
  |> groupBy("region")
  |> count("population")
  |> orderBy("population", desc: true)
  |> take(10);"#;

        let program = parse_src(src).expect("README 파이프라인 파싱 실패");
        match &program.stmts[0] {
            Stmt::VarDecl { var_name, ops, .. } => {
                assert_eq!(var_name, "blind_spots");
                assert_eq!(ops.len(), 7, "연산자 7개여야 함: {:?}", ops);

                assert_eq!(ops[0], PipelineOp::DropNull("income".into()));
                // filter(col("income") < 1_200_000)
                assert!(matches!(&ops[1], PipelineOp::Filter(Expr::BinOp { .. })));
                // filter(col("support") == false)
                assert!(matches!(&ops[2], PipelineOp::Filter(Expr::BinOp { .. })));

                assert_eq!(ops[3], PipelineOp::GroupBy("region".into()));
                assert_eq!(ops[4], PipelineOp::Count(Some("population".into())));
                assert_eq!(ops[5], PipelineOp::OrderBy { col: "population".into(), desc: true });
                assert_eq!(ops[6], PipelineOp::Take(10));
            }
            other => panic!("VarDecl 예상: {:?}", other),
        }
    }
}
