/// x1zzLang - AST 노드 정의

/// 표현식 노드
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// 식별자 참조
    Ident(String),
    /// 문자열 리터럴
    StringLit(String),
    /// 정수 리터럴
    IntLit(i64),
    /// 부동소수 리터럴
    FloatLit(f64),
    /// 이항 비교 연산 (lhs op rhs)
    BinOp {
        lhs: Box<Expr>,
        op: BinOpKind,
        rhs: Box<Expr>,
    },
}

/// 이항 연산자 종류
#[derive(Debug, Clone, PartialEq)]
pub enum BinOpKind {
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
}

/// 파이프라인 연산 단계
#[derive(Debug, Clone, PartialEq)]
pub enum PipelineOp {
    /// filter(<조건식>)
    Filter(Expr),
    /// select([col1, col2, ...])
    Select(Vec<String>),
    /// count
    Count,
}

/// 타입 선언의 필드 하나
#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub name: String,
    pub field_type: String,
}

/// 최상위 구문 노드
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// type <Name> { <fields> }
    TypeDecl {
        name: String,
        fields: Vec<StructField>,
    },
    /// load "<file>" :: <Schema> |> op1 |> op2 ...
    PipelineStream {
        file_path: String,
        schema_name: String,
        ops: Vec<PipelineOp>,
    },
}

/// 컴파일 단위 — 파일 전체 AST
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

impl Program {
    pub fn new() -> Self {
        Program { stmts: Vec::new() }
    }
}

impl Default for Program {
    fn default() -> Self {
        Program::new()
    }
}
