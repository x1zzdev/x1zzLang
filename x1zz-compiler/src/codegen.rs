/// x1zzLang - 코드 생성기 (완전 구현)
///
/// Phase-1 PoC: AST → Polars LazyFrame 연산 흐름 문자열로 매핑
///
/// 예시 출력:
///   [Schema] AirQuality { station: string, pm10: Option<float>, date: string }
///   [Pipeline] load("examples/seoul_error_2026.csv") :: AirQuality
///     |> filter(pm10 > 50)       // .filter(col("pm10").gt(50))
///     |> select([station, date]) // .select([col("station"), col("date")])
///     |> count                   // .count()
///     .collect()

use crate::ast::{BinOpKind, Expr, PipelineOp, Program, Stmt};
use crate::error::CompileResult;

pub struct Codegen {
    program: Program,
}

impl Codegen {
    pub fn new(program: Program) -> Self {
        Codegen { program }
    }

    /// 전체 Program AST를 읽어 Polars 흐름 주석 문자열 생성
    pub fn generate(&self) -> CompileResult<String> {
        let mut out = String::new();
        out.push_str("// ─────────────────────────────────────────────────────\n");
        out.push_str("// x1zzLang PoC — Polars LazyFrame 흐름 매핑\n");
        out.push_str("// ─────────────────────────────────────────────────────\n\n");

        for stmt in &self.program.stmts {
            out.push_str(&Self::describe_stmt(stmt));
            out.push('\n');
        }
        Ok(out)
    }

    /// 개별 Stmt를 Polars 흐름 문자열로 변환
    pub fn describe_stmt(stmt: &Stmt) -> String {
        match stmt {
            Stmt::TypeDecl { name, fields } => {
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|f| format!("  {}: {}", f.name, f.field_type))
                    .collect();
                format!(
                    "// [Schema] {} {{\n{}\n// }}\n",
                    name,
                    field_strs.join(",\n")
                )
            }
            Stmt::PipelineStream {
                file_path,
                schema_name,
                ops,
            } => Self::describe_pipeline(file_path, schema_name, ops),
        }
    }

    /// PipelineStream을 Polars LazyFrame 흐름 문자열로 변환
    pub fn describe_pipeline(
        file_path: &str,
        schema_name: &str,
        ops: &[PipelineOp],
    ) -> String {
        let mut lines: Vec<String> = Vec::new();

        // load 단계
        lines.push(format!(
            "LazyCsvReader::new(\"{}\").finish()  // load :: {}",
            file_path, schema_name
        ));

        // 각 파이프라인 연산 단계
        for op in ops {
            lines.push(Self::describe_op(op));
        }

        // collect (최종 실행 시점)
        lines.push(".collect()  // ← 모든 연산이 여기서 실행됨".into());

        lines.join("\n  ")
    }

    /// 개별 PipelineOp → Polars Rust 표현 문자열
    pub fn describe_op(op: &PipelineOp) -> String {
        match op {
            PipelineOp::Filter(expr) => {
                format!(
                    ".filter({})  // filter({})",
                    Self::expr_to_polars(expr),
                    Self::expr_to_xzz(expr)
                )
            }
            PipelineOp::Select(cols) => {
                let polars_cols: Vec<String> =
                    cols.iter().map(|c| format!("col(\"{}\")", c)).collect();
                let xzz_cols = cols.join(", ");
                format!(
                    ".select([{}])  // select([{}])",
                    polars_cols.join(", "),
                    xzz_cols
                )
            }
            PipelineOp::Count => ".count()  // count".to_string(),
        }
    }

    /// Expr → Polars Rust 표현식 문자열
    pub fn expr_to_polars(expr: &Expr) -> String {
        match expr {
            Expr::Ident(s)     => format!("col(\"{}\")", s),
            Expr::StringLit(s) => format!("lit(\"{}\")", s),
            Expr::IntLit(n)    => format!("lit({}i64)", n),
            Expr::FloatLit(f)  => format!("lit({}f64)", f),
            Expr::BinOp { lhs, op, rhs } => {
                let l = Self::expr_to_polars(lhs);
                let r = Self::expr_to_polars(rhs);
                match op {
                    BinOpKind::Eq    => format!("{}.eq({})", l, r),
                    BinOpKind::NotEq => format!("{}.neq({})", l, r),
                    BinOpKind::Lt    => format!("{}.lt({})", l, r),
                    BinOpKind::Gt    => format!("{}.gt({})", l, r),
                    BinOpKind::LtEq  => format!("{}.lt_eq({})", l, r),
                    BinOpKind::GtEq  => format!("{}.gt_eq({})", l, r),
                }
            }
        }
    }

    /// Expr → x1zzLang 소스 표현 문자열 (디버그 용)
    pub fn expr_to_xzz(expr: &Expr) -> String {
        match expr {
            Expr::Ident(s)     => s.clone(),
            Expr::StringLit(s) => format!("\"{}\"", s),
            Expr::IntLit(n)    => n.to_string(),
            Expr::FloatLit(f)  => f.to_string(),
            Expr::BinOp { lhs, op, rhs } => {
                let op_str = match op {
                    BinOpKind::Eq    => "==",
                    BinOpKind::NotEq => "!=",
                    BinOpKind::Lt    => "<",
                    BinOpKind::Gt    => ">",
                    BinOpKind::LtEq  => "<=",
                    BinOpKind::GtEq  => ">=",
                };
                format!(
                    "{} {} {}",
                    Self::expr_to_xzz(lhs),
                    op_str,
                    Self::expr_to_xzz(rhs)
                )
            }
        }
    }
}
