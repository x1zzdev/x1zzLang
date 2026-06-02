/// x1zzLang - AST 노드 정의 (v0.16)
///
/// [v0.16 변경사항]
///   - Expr::BoolLit(bool) 추가 — true/false 리터럴
///   - FillNullValue 열거형 추가 — fillNull 채우기 값
///   - PipelineOp::Count → Count(Option<String>) 으로 변경
///      (None = 전체 행 수 / Some(col) = 그룹 내 컬럼 카운트)
///   - 신규 PipelineOp 변형 9종:
///      GroupBy, Sum, Mean, Min, Max, OrderBy, Take, DropNull, FillNull

/// 표현식 노드
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// 식별자 참조 (변수명 또는 컬럼명)
    Ident(String),
    /// 문자열 리터럴
    StringLit(String),
    /// 정수 리터럴
    IntLit(i64),
    /// 부동소수 리터럴
    FloatLit(f64),
    /// 불리언 리터럴 (true / false)
    BoolLit(bool),
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

/// fillNull 채우기 값 종류
#[derive(Debug, Clone, PartialEq)]
pub enum FillNullValue {
    /// 정수 채우기 값
    Int(i64),
    /// 부동소수 채우기 값
    Float(f64),
    /// 문자열 채우기 값
    Str(String),
}

/// 파이프라인 연산 단계
#[derive(Debug, Clone, PartialEq)]
pub enum PipelineOp {
    /// filter(<조건식>)
    Filter(Expr),
    /// select([col1, col2, ...])
    Select(Vec<String>),
    /// count  (None: 전체 행 수용 플래그) / count("col")  (Some: 그룹 집계)
    Count(Option<String>),
    /// groupBy("col")  — 이후 Sum/Mean/Min/Max/Count(Some) 와 쌍으로 사용
    GroupBy(String),
    /// sum("col")  — 단독 또는 groupBy 뒤에 사용
    Sum(String),
    /// mean("col")  — 단독 또는 groupBy 뒤에 사용
    Mean(String),
    /// min("col")  — 단독 또는 groupBy 뒤에 사용
    Min(String),
    /// max("col")  — 단독 또는 groupBy 뒤에 사용
    Max(String),
    /// orderBy("col", desc: true/false)
    OrderBy { col: String, desc: bool },
    /// take(n)  — 상위 n 행만 유지
    Take(i64),
    /// dropNull("col")  — 해당 컬럼이 null인 행 제거
    DropNull(String),
    /// fillNull("col", value)  — 해당 컬럼의 null을 value로 채우기
    FillNull { col: String, value: FillNullValue },
}

/// 파이프라인의 소스 (데이터 원천)
#[derive(Debug, Clone, PartialEq)]
pub enum PipelineSource {
    /// load("파일경로") :: SchemaName
    Load {
        file_path: String,
        schema_name: String,
    },
    /// 이미 선언된 변수를 참조
    VarRef(String),
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
    /// type <Name> = { <fields> }
    TypeDecl {
        name: String,
        fields: Vec<StructField>,
    },
    /// (mut)? v <name> = <source> |> op1 |> op2 ...
    VarDecl {
        var_name: String,
        is_mut: bool,
        source: PipelineSource,
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
