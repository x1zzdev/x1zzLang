/// x1zzLang - AST 노드 정의 (v0.19)
///
/// [v0.19 변경사항]
///   - ChartType 열거형 추가: Bar, Line, Pie, Scatter
///   - ChartConfig 구조체 추가 — chart { ... } 블록 설정
///   - PipelineOp::Chart(ChartConfig) 추가 — 파이프라인 시각화 연산
///
/// [v0.16 변경사항]
///   - Expr::BoolLit(bool) 추가 — true/false 리터럴
///   - FillNullValue 열거형 추가 — fillNull 채우기 값
///   - PipelineOp::Count → Count(Option<String>) 으로 변경
///      (None = 전체 행 수 / Some(col) = 그룹 내 컬럼 카운트)
///   - 신규 PipelineOp 변형 9종:
///      GroupBy, Sum, Mean, Min, Max, OrderBy, Take, DropNull, FillNull
///   - BinOpKind 산술 연산 추가: Add, Sub, Mul, Div
///   - PipelineOp::Join { other, on, how } 추가
///   - PipelineOp::WithColumn { name, expr } 추가
///   - JoinHow 열거형 추가

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
    /// 이항 연산 (lhs op rhs) — 비교 및 산술 연산 포함
    BinOp {
        lhs: Box<Expr>,
        op: BinOpKind,
        rhs: Box<Expr>,
    },
}

/// 이항 연산자 종류 (비교 + 산술)
#[derive(Debug, Clone, PartialEq)]
pub enum BinOpKind {
    // ── 비교 연산자 ──────────────────────
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    // ── 산술 연산자 (v0.16+) ─────────────
    Add,
    Sub,
    Mul,
    Div,
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

/// join 방식
#[derive(Debug, Clone, PartialEq)]
pub enum JoinHow {
    Inner,
    Left,
    Outer,
    Cross,
}

impl Default for JoinHow {
    fn default() -> Self {
        JoinHow::Inner
    }
}

impl JoinHow {
    /// 소문자 문자열에서 파싱
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "inner" => Some(JoinHow::Inner),
            "left" => Some(JoinHow::Left),
            "outer" => Some(JoinHow::Outer),
            "cross" => Some(JoinHow::Cross),
            _ => None,
        }
    }

    pub fn as_polars_str(&self) -> &'static str {
        match self {
            JoinHow::Inner => "JoinType::Inner",
            JoinHow::Left => "JoinType::Left",
            JoinHow::Outer => "JoinType::Full",
            JoinHow::Cross => "JoinType::Cross",
        }
    }
}

// ── v0.19 시각화 타입 ──────────────────────────────────────────────────────────

/// 차트 종류 (MVP: bar / line / pie / scatter)
#[derive(Debug, Clone, PartialEq)]
pub enum ChartType {
    Bar,
    Line,
    Pie,
    Scatter,
}

impl ChartType {
    /// 식별자 문자열에서 파싱
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "bar" => Some(ChartType::Bar),
            "line" => Some(ChartType::Line),
            "pie" => Some(ChartType::Pie),
            "scatter" => Some(ChartType::Scatter),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ChartType::Bar => "bar",
            ChartType::Line => "line",
            ChartType::Pie => "pie",
            ChartType::Scatter => "scatter",
        }
    }
}

/// chart { ... } 블록 설정값
#[derive(Debug, Clone, PartialEq)]
pub struct ChartConfig {
    pub chart_type: ChartType,
    pub title: Option<String>,
    /// x축 컬럼명 (bar, line, scatter 용)
    pub x: Option<String>,
    /// y축 컬럼명 (bar, line, scatter 용)
    pub y: Option<String>,
    /// 레이블 컬럼명 (pie 용)
    pub label: Option<String>,
    /// 값 컬럼명 (pie 용)
    pub value: Option<String>,
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
    /// join(other_var, on: "key") 또는 join(other_var, on: ["k1","k2"], how: "left")
    /// join(other_var, left_on: "station", right_on: "adm_name") — 다른 키명 조인 (v0.21)
    Join {
        other: String,
        /// left 테이블 조인 키 (on: 사용 시 on_keys, left_on: 사용 시 left_on_keys)
        left_on: Vec<String>,
        /// right 테이블 조인 키 (on: 사용 시 on_keys, right_on: 사용 시 right_on_keys)
        right_on: Vec<String>,
        how: JoinHow,
    },
    /// withColumn("new_col", expr)  — 새로운 컬럼 추가/변환
    WithColumn { name: String, expr: Expr },
    /// chart { type: ..., x: ..., y: ..., title: "..." }  — 파이프라인 시각화 (v0.19)
    Chart(ChartConfig),
    /// cast("col", "float")  — 컬럼 타입을 DSL 레벨에서 명시적으로 캐스팅 (v0.20)
    ///
    /// 지원 타입 문자열: "float", "int", "str", "bool"
    /// 런타임은 이 연산만을 실행하며, 어떤 컬럼을 캐스팅할지 스스로 추론하지 않는다.
    Cast { col: String, to_type: String },
    /// rename("old_name", "new_name") — 컬럼 이름 변경 (v0.21)
    Rename { old_name: String, new_name: String },
    /// replace("col", ".", "") — 문자열 치환 (v0.21)
    /// col 컬럼의 모든 from 문자열을 to 로 치환
    Replace {
        col: String,
        from: String,
        to: String,
    },
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
    /// expression statement: 변수에 할당하지 않고 파이프라인 실행 (결과 버림)
    /// 예: final_result |> chart({...})
    ExprStmt {
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
