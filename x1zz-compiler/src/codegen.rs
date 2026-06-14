/// x1zzLang - 코드 생성기 (v0.16)
///
/// Program AST → Polars LazyFrame 흐름 문자열 생성.
///
/// [v0.16 변경사항]
///   - BoolLit 표현식 지원 (true/false)
///   - Count(None) / Count(Some(col)) 구분
///   - 신규 연산자 출력: GroupBy, Sum, Mean, Min, Max, OrderBy, Take, DropNull, FillNull
///   - Join 연산자: .join(..., ..., JoinArgs::new(JoinType::Inner)) 매핑
///   - WithColumn 연산자: .with_columns([expr.alias("name")]) 매핑
///   - 산술 연산자: Add/Sub/Mul/Div → .add()/.sub()/.mul()/.div()
use crate::ast::{BinOpKind, Expr, FillNullValue, PipelineOp, PipelineSource, Program, Stmt};

/// 코드 생성기 — 유닛 구조체
pub struct Codegen;

impl Codegen {
    pub fn new() -> Self {
        Codegen
    }

    // ── 최상위 진입점 ─────────────────────────────────────────────────────────

    /// Program AST 전체를 Polars 흐름 문자열로 생성
    pub fn generate(program: &Program) -> String {
        let mut out = String::new();
        out.push_str("// ═══════════════════════════════════════════════════════\n");
        out.push_str("// x1zzLang → Polars LazyFrame 흐름 매핑\n");
        out.push_str("// ═══════════════════════════════════════════════════════\n\n");

        for (i, stmt) in program.stmts.iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            out.push_str(&Self::emit_stmt(stmt));
        }
        out
    }

    // ── Stmt 변환 ─────────────────────────────────────────────────────────────

    fn emit_stmt(stmt: &Stmt) -> String {
        match stmt {
            Stmt::TypeDecl { name, fields } => {
                let mut s = format!("// [Schema] {}\n", name);
                for f in fields {
                    s.push_str(&format!("//   {:<12} : {}\n", f.name, f.field_type));
                }
                s
            }
            Stmt::VarDecl {
                var_name,
                is_mut,
                source,
                ops,
            } => Self::emit_var_decl(var_name, *is_mut, source, ops),
            Stmt::ExprStmt { source, ops } => Self::emit_expr_stmt(source, ops),
        }
    }

    // ── VarDecl 변환 ──────────────────────────────────────────────────────────

    fn emit_var_decl(
        var_name: &str,
        is_mut: bool,
        source: &PipelineSource,
        ops: &[PipelineOp],
    ) -> String {
        let mut lines: Vec<String> = Vec::new();

        // 주석 헤더
        let source_comment = match source {
            PipelineSource::Load {
                file_path,
                schema_name,
            } => {
                format!("load(\"{}\") :: {}", file_path, schema_name)
            }
            PipelineSource::VarRef(name) => {
                format!("{} (varref)", name)
            }
        };
        let mut_kw = if is_mut { "mut " } else { "" };
        lines.push(format!(
            "// [VarDecl] {}v {} = {}",
            mut_kw, var_name, source_comment
        ));

        // 소스 코드 생성
        match source {
            PipelineSource::Load {
                file_path,
                schema_name,
            } => {
                lines.push(format!(
                    "let {} = LazyCsvReader::new(\"{}\")  // :: {}",
                    var_name, file_path, schema_name
                ));
                lines.push("  .with_has_header(true)".into());
                lines.push("  .finish()?".into());
            }
            PipelineSource::VarRef(src_var) => {
                lines.push(format!("let {} = {}.clone().lazy()", var_name, src_var));
            }
        }

        // 파이프라인 각 단계
        for op in ops {
            lines.push(Self::emit_op(op));
        }

        // collect — Lazy 실행 시점
        lines.push(format!(
            "  .collect()?;  // ← {}: 모든 연산 일괄 실행",
            var_name
        ));

        lines.join("\n")
    }

    // ── ExprStmt 변환 ─────────────────────────────────────────────────────────

    fn emit_expr_stmt(source: &PipelineSource, ops: &[PipelineOp]) -> String {
        let mut lines: Vec<String> = Vec::new();

        let source_comment = match source {
            PipelineSource::Load {
                file_path,
                schema_name,
            } => {
                format!("load(\"{}\") :: {}", file_path, schema_name)
            }
            PipelineSource::VarRef(name) => {
                format!("{} (varref)", name)
            }
        };
        lines.push(format!("// [ExprStmt] source = {}", source_comment));

        match source {
            PipelineSource::Load {
                file_path,
                schema_name,
            } => {
                lines.push(format!(
                    "let _expr_result = LazyCsvReader::new(\"{}\")  // :: {}",
                    file_path, schema_name
                ));
                lines.push("  .with_has_header(true)".into());
                lines.push("  .finish()?".into());
            }
            PipelineSource::VarRef(src_var) => {
                lines.push(format!("let _expr_result = {}.clone().lazy()", src_var));
            }
        }

        for op in ops {
            lines.push(Self::emit_op(op));
        }

        lines.push("  .collect()?;  // ← expression statement 실행".to_string());
        lines.join("\n")
    }

    // ── Op 변환 ───────────────────────────────────────────────────────────────

    fn emit_op(op: &PipelineOp) -> String {
        match op {
            // ── 기존 ──────────────────────────────────────────────────────────
            PipelineOp::Filter(expr) => {
                format!(
                    "  .filter({})  // |> filter({})",
                    Self::expr_to_polars(expr),
                    Self::expr_to_xzz(expr)
                )
            }
            PipelineOp::Select(cols) => {
                let polars: Vec<String> = cols.iter().map(|c| format!("col(\"{}\")", c)).collect();
                let xzz = cols.join(", ");
                format!(
                    "  .select([{}])  // |> select([{}])",
                    polars.join(", "),
                    xzz
                )
            }
            PipelineOp::Count(None) => "  // |> count  →  df.height() 로 행 수 확인".to_string(),
            PipelineOp::Count(Some(col)) => {
                format!(
                    "  .agg([col(\"{}\").count()])  // |> count(\"{}\")",
                    col, col
                )
            }

            // ── v0.16 집계 ────────────────────────────────────────────────────
            PipelineOp::GroupBy(group_col) => {
                format!(
                    "  .group_by([col(\"{}\")])  // |> groupBy(\"{}\")",
                    group_col, group_col
                )
            }
            PipelineOp::Sum(agg_col) => {
                format!(
                    "  .agg([col(\"{}\").sum()])  // |> sum(\"{}\")",
                    agg_col, agg_col
                )
            }
            PipelineOp::Mean(agg_col) => {
                format!(
                    "  .agg([col(\"{}\").mean()])  // |> mean(\"{}\")",
                    agg_col, agg_col
                )
            }
            PipelineOp::Min(agg_col) => {
                format!(
                    "  .agg([col(\"{}\").min()])  // |> min(\"{}\")",
                    agg_col, agg_col
                )
            }
            PipelineOp::Max(agg_col) => {
                format!(
                    "  .agg([col(\"{}\").max()])  // |> max(\"{}\")",
                    agg_col, agg_col
                )
            }

            // ── v0.16 정렬 / 슬라이싱 ─────────────────────────────────────────
            PipelineOp::OrderBy { col, desc } => {
                format!(
                    "  .sort([\"{}\"], SortMultipleOptions::default().with_order_descending({}))  // |> orderBy(\"{}\", desc: {})",
                    col, desc, col, desc
                )
            }
            PipelineOp::Take(n) => {
                format!("  .limit({})  // |> take({})", n, n)
            }

            // ── v0.16 Null 처리 ────────────────────────────────────────────────
            PipelineOp::DropNull(drop_col) => {
                format!(
                    "  .drop_nulls(Some(vec![col(\"{}\")]))  // |> dropNull(\"{}\")",
                    drop_col, drop_col
                )
            }
            PipelineOp::FillNull { col, value } => {
                let lit_str = match value {
                    FillNullValue::Int(n) => format!("lit({}i64)", n),
                    FillNullValue::Float(f) => format!("lit({}f64)", f),
                    FillNullValue::Str(s) => format!("lit(\"{}\")", s),
                };
                format!(
                    "  .with_columns([col(\"{}\").fill_null({})])  // |> fillNull(\"{}\", ...)",
                    col, lit_str, col
                )
            }

            // ── v0.16+ / v0.21 Join ──────────────────────────────────────────
            PipelineOp::Join {
                other,
                left_on,
                right_on,
                how,
            } => {
                let left_cols: Vec<String> =
                    left_on.iter().map(|k| format!("col(\"{}\")", k)).collect();
                let right_cols: Vec<String> =
                    right_on.iter().map(|k| format!("col(\"{}\")", k)).collect();
                let left_str = left_cols.join(", ");
                let right_str = right_cols.join(", ");
                format!(
                    "  .join({}.lazy(), [{}], [{}], JoinArgs::new({}))  // |> join({}, left_on: {:?}, right_on: {:?}, how: {:?})",
                    other,
                    left_str,
                    right_str,
                    how.as_polars_str(),
                    other,
                    left_on,
                    right_on,
                    how
                )
            }

            // ── v0.16+ WithColumn ─────────────────────────────────────────────
            PipelineOp::WithColumn { name, expr } => {
                format!(
                    "  .with_columns([{}.alias(\"{}\")])  // |> withColumn(\"{}\", {})",
                    Self::expr_to_polars(expr),
                    name,
                    name,
                    Self::expr_to_xzz(expr)
                )
            }

            // ── Chart: codegen 미지원 (런타임에서만 처리) ─────────────────────
            PipelineOp::Chart(config) => {
                format!(
                    "  // |> chart {{ type: {} }}  →  [x1zz:chart] JSON 출력",
                    config.chart_type.as_str()
                )
            }

            // ── v0.20 Cast ───────────────────────────────────────────────────
            PipelineOp::Cast { col, to_type } => {
                let polars_type = match to_type.as_str() {
                    "float" => "DataType::Float64",
                    "int" => "DataType::Int64",
                    "str" => "DataType::String",
                    "bool" => "DataType::Boolean",
                    other => other,
                };
                format!(
                    "  .with_columns([col(\"{}\").cast({})])  // |> cast(\"{}\", \"{}\")",
                    col, polars_type, col, to_type
                )
            }

            // ── Rename ───────────────────────────────────────────────────────
            PipelineOp::Rename { old_name, new_name } => {
                format!(
                    "  .rename([\"{}\"], [\"{}\"], false)  // |> rename(\"{}\", \"{}\")",
                    old_name, new_name, old_name, new_name
                )
            }

            // ── Replace ──────────────────────────────────────────────────────
            PipelineOp::Replace { col, from, to } => {
                format!(
                    "  .with_columns([col(\"{}\").str().replace(lit(\"{}\"), lit(\"{}\"), false).alias(\"{}\")])  // |> replace(\"{}\", \"{}\", \"{}\")",
                    col, from, to, col, col, from, to
                )
            }
        }
    }

    // ── 표현식 → Polars Rust ──────────────────────────────────────────────────

    pub fn expr_to_polars(expr: &Expr) -> String {
        match expr {
            Expr::Ident(s) => format!("col(\"{}\")", s),
            Expr::StringLit(s) => format!("lit(\"{}\")", s),
            Expr::IntLit(n) => format!("lit({}i64)", n),
            Expr::FloatLit(f) => format!("lit({}f64)", f),
            Expr::BoolLit(b) => format!("lit({})", b),
            Expr::BinOp { lhs, op, rhs } => {
                let l = Self::expr_to_polars(lhs);
                let r = Self::expr_to_polars(rhs);
                match op {
                    BinOpKind::Eq => format!("{}.eq({})", l, r),
                    BinOpKind::NotEq => format!("{}.neq({})", l, r),
                    BinOpKind::Lt => format!("{}.lt({})", l, r),
                    BinOpKind::Gt => format!("{}.gt({})", l, r),
                    BinOpKind::LtEq => format!("{}.lt_eq({})", l, r),
                    BinOpKind::GtEq => format!("{}.gt_eq({})", l, r),
                    // ── 산술 연산자 (v0.16+) ──────────────────
                    BinOpKind::Add => format!("{}.add({})", l, r),
                    BinOpKind::Sub => format!("{}.sub({})", l, r),
                    BinOpKind::Mul => format!("{}.mul({})", l, r),
                    BinOpKind::Div => format!("{}.div({})", l, r),
                }
            }
        }
    }

    // ── 표현식 → x1zzLang 소스 표현 ──────────────────────────────────────────

    pub fn expr_to_xzz(expr: &Expr) -> String {
        match expr {
            Expr::Ident(s) => s.clone(),
            Expr::StringLit(s) => format!("\"{}\"", s),
            Expr::IntLit(n) => n.to_string(),
            Expr::FloatLit(f) => f.to_string(),
            Expr::BoolLit(b) => b.to_string(),
            Expr::BinOp { lhs, op, rhs } => {
                let op_str = match op {
                    BinOpKind::Eq => "==",
                    BinOpKind::NotEq => "!=",
                    BinOpKind::Lt => "<",
                    BinOpKind::Gt => ">",
                    BinOpKind::LtEq => "<=",
                    BinOpKind::GtEq => ">=",
                    BinOpKind::Add => "+",
                    BinOpKind::Sub => "-",
                    BinOpKind::Mul => "*",
                    BinOpKind::Div => "/",
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

impl Default for Codegen {
    fn default() -> Self {
        Codegen::new()
    }
}
