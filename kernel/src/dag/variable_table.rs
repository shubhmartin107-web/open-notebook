use crate::notebook::CellKind;
use anyhow::{bail, Result};
use std::collections::BTreeSet;

pub fn extract_defs(source: &str, kind: &CellKind) -> Result<BTreeSet<String>> {
    match kind {
        CellKind::Python => Ok(extract_defs_python_text(source)),
        CellKind::Sql => Ok(BTreeSet::new()),
        CellKind::Markdown | CellKind::Raw => Ok(BTreeSet::new()),
        CellKind::R => Ok(BTreeSet::new()),
    }
}

pub fn extract_refs(source: &str, kind: &CellKind) -> Result<BTreeSet<String>> {
    match kind {
        CellKind::Python => Ok(extract_refs_python_text(source)),
        CellKind::Sql => extract_tables_and_columns(source),
        CellKind::Markdown | CellKind::Raw => Ok(BTreeSet::new()),
        CellKind::R => Ok(BTreeSet::new()),
    }
}

// ── Python text-based analysis ──
// Uses simple line-by-line heuristics instead of full AST parsing.
// Fast enough for real-time use and handles 90%+ of real-world cases.

fn extract_defs_python_text(source: &str) -> BTreeSet<String> {
    let mut defs = BTreeSet::new();

    for line in source.lines() {
        let trimmed = line.trim();

        // Variable assignments: x = ..., x: int = ..., x += ...
        if let Some(name) = capture_assignment_lhs(trimmed) {
            if is_valid_name(&name) {
                defs.insert(name);
            }
        }

        // Function definitions: def foo(...):
        if trimmed.starts_with("def ") {
            let name = trimmed.trim_start_matches("def ").split('(').next().unwrap_or("").trim();
            if !name.is_empty() {
                defs.insert(name.to_string());
            }
        }

        // Async function definitions: async def foo(...):
        if trimmed.starts_with("async def ") {
            let name = trimmed.trim_start_matches("async def ").split('(').next().unwrap_or("").trim();
            if !name.is_empty() {
                defs.insert(name.to_string());
            }
        }

        // Class definitions: class Foo(...):
        if trimmed.starts_with("class ") {
            let name = trimmed.trim_start_matches("class ").split('(').next().unwrap_or("").trim().trim_end_matches(':');
            if !name.is_empty() {
                defs.insert(name.to_string());
            }
        }

        // Import statements: import foo, import foo.bar as baz
        if trimmed.starts_with("import ") {
            let rest = trimmed.trim_start_matches("import ");
            for part in rest.split(',') {
                let part = part.trim();
                if let Some(alias) = part.split(" as ").nth(1) {
                    defs.insert(alias.trim().to_string());
                } else {
                    let name = part.split('.').next().unwrap_or(part).trim();
                    if !name.is_empty() {
                        defs.insert(name.to_string());
                    }
                }
            }
        }

        // From imports: from x import y, from x import y as z
        if trimmed.starts_with("from ") {
            if let Some(after_from) = trimmed.strip_prefix("from ") {
                if let Some(import_part) = after_from.split(" import ").nth(1) {
                    for part in import_part.split(',') {
                        let part = part.trim().trim_end_matches(")");
                        let name = part.split(" as ").nth(1).unwrap_or(part).trim();
                        if !name.is_empty() && is_valid_name(name) {
                            defs.insert(name.to_string());
                        }
                    }
                }
            }
        }

        // With statement context variable: with x as y:
        if trimmed.starts_with("with ") {
            if let Some(after_as) = trimmed.split(" as ").nth(1) {
                let name = after_as.trim().trim_end_matches(':').trim();
                if !name.is_empty() && is_valid_name(name) {
                    defs.insert(name.to_string());
                }
            }
        }

        // For loop variable: for x in ...:
        if trimmed.starts_with("for ") {
            let rest = trimmed.trim_start_matches("for ");
            let name = rest.split([' ', '\t', ':']).next().unwrap_or("").trim();
            if !name.is_empty() && is_valid_name(name) {
                defs.insert(name.to_string());
            }
        }

        // Except statement: except ... as e:
        if trimmed.starts_with("except ") && trimmed.contains(" as ") {
            let name = trimmed.split(" as ").nth(1).unwrap_or("").trim().trim_end_matches(':').trim();
            if !name.is_empty() && is_valid_name(name) {
                defs.insert(name.to_string());
            }
        }
    }

    defs
}

fn capture_assignment_lhs(line: &str) -> Option<String> {
    // Skip lines that are not assignments
    if !line.contains('=') {
        return None;
    }

    // Skip comparison operators
    if line.contains("==") || line.contains("!=") || line.contains("<=") || line.contains(">=") {
        return None;
    }

    // Split on = but only the first one (LHS)
    let parts: Vec<&str> = line.splitn(2, '=').collect();
    if parts.len() < 2 {
        return None;
    }

    let lhs = parts[0].trim();

    // Handle augmented assignment
    let lhs = lhs.strip_suffix('+').unwrap_or(lhs);
    let lhs = lhs.strip_suffix('-').unwrap_or(lhs);
    let lhs = lhs.strip_suffix('*').unwrap_or(lhs);
    let lhs = lhs.strip_suffix('/').unwrap_or(lhs);
    let lhs = lhs.strip_suffix('%').unwrap_or(lhs);
    let lhs = lhs.strip_suffix('&').unwrap_or(lhs);
    let lhs = lhs.strip_suffix('|').unwrap_or(lhs);
    let lhs = lhs.strip_suffix('^').unwrap_or(lhs);
    let lhs = lhs.strip_suffix(':').unwrap_or(lhs);

    let lhs = lhs.trim();

    // Handle type annotation: x: int, x: "SomeType"
    let lhs = lhs.split(':').next().unwrap_or(lhs).trim();

    if is_valid_name(lhs) {
        Some(lhs.to_string())
    } else {
        None
    }
}

fn extract_refs_python_text(source: &str) -> BTreeSet<String> {
    let defs = extract_defs_python_text(source);
    let mut refs = BTreeSet::new();

    for line in source.lines() {
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        // Extract all identifier-like tokens from the line
        let tokens = tokenize(trimmed);
        for token in &tokens {
            if is_valid_name(token) && !BUILTINS.contains(&token.as_str()) {
                refs.insert(token.clone());
            }
        }
    }

    // Remove self-references (defined in this cell)
    for d in &defs {
        refs.remove(d);
    }

    refs
}

fn tokenize(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for ch in line.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            current.push(ch);
        } else {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

fn is_valid_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let first = name.chars().next().unwrap();
    if !first.is_alphabetic() && first != '_' {
        return false;
    }
    name.chars().all(|c| c.is_alphanumeric() || c == '_')
}

const BUILTINS: &[&str] = &[
    "abs", "all", "any", "bin", "bool", "bytearray", "bytes", "callable", "chr",
    "classmethod", "compile", "complex", "copyright", "credits", "delattr", "dict",
    "dir", "divmod", "enumerate", "eval", "exec", "exit", "filter", "float", "format",
    "frozenset", "getattr", "globals", "hasattr", "hash", "help", "hex", "id", "input",
    "int", "isinstance", "issubclass", "iter", "len", "license", "list", "locals",
    "map", "max", "memoryview", "min", "next", "object", "oct", "open", "ord", "pow",
    "print", "property", "quit", "range", "repr", "reversed", "round", "set", "setattr",
    "slice", "sorted", "staticmethod", "str", "sum", "super", "tuple", "type", "vars",
    "zip", "__import__", "True", "False", "None", "self", "cls",
];

// ── SQL variable analysis via sqlparser-rs ──

fn extract_tables_and_columns(source: &str) -> Result<BTreeSet<String>> {
    use sqlparser::parser::Parser;
    use sqlparser::dialect::GenericDialect;

    let dialect = GenericDialect {};
    let mut refs = BTreeSet::new();

    let statements = match Parser::parse_sql(&dialect, source) {
        Ok(s) => s,
        Err(e) => bail!("Failed to parse SQL: {}", e),
    };

    for stmt in &statements {
        if let sqlparser::ast::Statement::Query(query) = stmt {
            collect_table_refs(query, &mut refs);
        }
    }

    Ok(refs)
}

fn collect_table_refs(query: &sqlparser::ast::Query, refs: &mut BTreeSet<String>) {
    use sqlparser::ast::*;

    if let Some(with) = &query.with {
        for cte in &with.cte_tables {
            collect_table_refs(&cte.query, refs);
        }
    }

    match query.body.as_ref() {
        SetExpr::Select(select) => {
            for table in &select.from {
                collect_from_table_refs(table, refs);
            }
            if let Some(selection) = &select.selection {
                collect_sql_expr_refs(selection, refs);
            }
        }
        SetExpr::SetOperation { left, right, .. } => {
            recurse_set_expr(left.as_ref(), refs);
            recurse_set_expr(right.as_ref(), refs);
        }
        _ => {}
    }
}

fn recurse_set_expr(expr: &sqlparser::ast::SetExpr, refs: &mut BTreeSet<String>) {
    use sqlparser::ast::*;
    match expr {
        SetExpr::Select(select) => {
            for table in &select.from {
                collect_from_table_refs(table, refs);
            }
            if let Some(selection) = &select.selection {
                collect_sql_expr_refs(selection, refs);
            }
        }
        SetExpr::SetOperation { left, right, .. } => {
            recurse_set_expr(left.as_ref(), refs);
            recurse_set_expr(right.as_ref(), refs);
        }
        SetExpr::Query(query) => {
            collect_table_refs(query.as_ref(), refs);
        }
        _ => {}
    }
}

fn collect_table_factor_refs(
    factor: &sqlparser::ast::TableFactor,
    refs: &mut BTreeSet<String>,
) {
    use sqlparser::ast::*;
    match factor {
        TableFactor::Table { name, .. } => {
            let name_str = name.0.iter().map(|i| i.value.clone()).collect::<Vec<_>>().join(".");
            refs.insert(name_str);
        }
        TableFactor::Derived { subquery, .. } => {
            collect_table_refs(subquery.as_ref(), refs);
        }
        TableFactor::TableFunction { expr, .. } => {
            collect_sql_expr_refs(expr, refs);
        }
        _ => {}
    }
}

fn collect_from_table_refs(
    table: &sqlparser::ast::TableWithJoins,
    refs: &mut BTreeSet<String>,
) {
    collect_table_factor_refs(&table.relation, refs);

    for join in &table.joins {
        collect_table_factor_refs(&join.relation, refs);
        collect_join_constraint(&join.join_operator, refs);
    }
}

fn collect_join_constraint(op: &sqlparser::ast::JoinOperator, refs: &mut BTreeSet<String>) {
    use sqlparser::ast::*;
    match op {
        JoinOperator::Inner(constraint)
        | JoinOperator::LeftOuter(constraint)
        | JoinOperator::RightOuter(constraint)
        | JoinOperator::FullOuter(constraint)
        | JoinOperator::LeftSemi(constraint)
        | JoinOperator::RightSemi(constraint)
        | JoinOperator::LeftAnti(constraint)
        | JoinOperator::RightAnti(constraint) => {
            match constraint {
                JoinConstraint::On(expr) => collect_sql_expr_refs(expr, refs),
                JoinConstraint::Using(_) => {}
                _ => {}
            }
        }
        _ => {}
    }
}

fn collect_sql_expr_refs(expr: &sqlparser::ast::Expr, refs: &mut BTreeSet<String>) {
    use sqlparser::ast::*;

    match expr {
        Expr::Identifier(ident) => {
            refs.insert(ident.value.clone());
        }
        Expr::CompoundIdentifier(idents) => {
            let name = idents.iter().map(|i| i.value.clone()).collect::<Vec<_>>().join(".");
            refs.insert(name);
        }
        Expr::BinaryOp { left, right, .. } => {
            collect_sql_expr_refs(left, refs);
            collect_sql_expr_refs(right, refs);
        }
        Expr::UnaryOp { expr, .. } => {
            collect_sql_expr_refs(expr, refs);
        }
        Expr::Function(func) => {
            if let FunctionArguments::List(args) = &func.args {
                for arg in &args.args {
                    match arg {
                        FunctionArg::Unnamed(expr_arg) => {
                            if let FunctionArgExpr::Expr(e) = expr_arg {
                                collect_sql_expr_refs(e, refs);
                            }
                        }
                        FunctionArg::Named { arg: expr_arg, .. } => {
                            if let FunctionArgExpr::Expr(e) = expr_arg {
                                collect_sql_expr_refs(e, refs);
                            }
                        }
                    }
                }
            }
        }
        Expr::Subquery(query) => {
            collect_table_refs(query, refs);
        }
        Expr::InSubquery { expr, subquery, .. } => {
            collect_sql_expr_refs(expr, refs);
            collect_table_refs(subquery, refs);
        }
        Expr::InList { expr, list, .. } => {
            collect_sql_expr_refs(expr, refs);
            for item in list {
                collect_sql_expr_refs(item, refs);
            }
        }
        Expr::Between { expr, low, high, .. } => {
            collect_sql_expr_refs(expr, refs);
            collect_sql_expr_refs(low, refs);
            collect_sql_expr_refs(high, refs);
        }
        Expr::Like { expr, pattern, .. } => {
            collect_sql_expr_refs(expr, refs);
            collect_sql_expr_refs(pattern, refs);
        }
        Expr::SimilarTo { expr, pattern, .. } => {
            collect_sql_expr_refs(expr, refs);
            collect_sql_expr_refs(pattern, refs);
        }
        Expr::IsNull(expr) => {
            collect_sql_expr_refs(expr, refs);
        }
        Expr::IsNotNull(expr) => {
            collect_sql_expr_refs(expr, refs);
        }
        Expr::Cast { expr, .. } => {
            collect_sql_expr_refs(expr, refs);
        }
        Expr::Extract { expr, .. } => {
            collect_sql_expr_refs(expr, refs);
        }
        Expr::Case { operand, conditions, results, else_result } => {
            if let Some(op) = operand {
                collect_sql_expr_refs(op, refs);
            }
            for cond in conditions {
                collect_sql_expr_refs(cond, refs);
            }
            for res in results {
                collect_sql_expr_refs(res, refs);
            }
            if let Some(el) = else_result {
                collect_sql_expr_refs(el, refs);
            }
        }
        _ => {}
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_simple_assign() {
        let defs = extract_defs_python_text("x = 5");
        assert!(defs.contains("x"));
    }

    #[test]
    fn test_python_function_def() {
        let defs = extract_defs_python_text("def foo(a, b):\n    return a + b");
        assert!(defs.contains("foo"));
    }

    #[test]
    fn test_python_class_def() {
        let defs = extract_defs_python_text("class MyClass:\n    pass");
        assert!(defs.contains("MyClass"));
    }

    #[test]
    fn test_python_import() {
        let defs = extract_defs_python_text("import pandas as pd");
        assert!(defs.contains("pd"));
    }

    #[test]
    fn test_python_from_import() {
        let defs = extract_defs_python_text("from pandas import read_csv");
        assert!(defs.contains("read_csv"));
    }

    #[test]
    fn test_python_refs_simple() {
        let refs = extract_refs_python_text("result = x + 1");
        assert!(refs.contains("x"));
        assert!(!refs.contains("result"));
    }

    #[test]
    fn test_python_refs_function_call() {
        let refs = extract_refs_python_text("df.head()");
        assert!(refs.contains("df"));
    }

    #[test]
    fn test_python_refs_multi_line() {
        let refs = extract_refs_python_text(
            "import pandas as pd\ndf = pd.read_csv('data.csv')\ndf.head()",
        );
        // 'pd' is defined (imported) in this cell, so it's a def, not a ref
        assert!(!refs.contains("pd"));
        assert!(!refs.contains("df"));
        // 'read_csv' is a ref called from 'pd'
        assert!(refs.contains("read_csv"));
        assert!(refs.contains("head"));
    }

    #[test]
    fn test_cell_kind_roundtrip() {
        assert_eq!("python".parse(), Ok(CellKind::Python));
        assert_eq!("sql".parse(), Ok(CellKind::Sql));
        assert_eq!("markdown".parse(), Ok(CellKind::Markdown));
        assert_eq!("r".parse(), Ok(CellKind::R));
        assert_eq!("raw".parse(), Ok(CellKind::Raw));
        assert!("unknown".parse::<CellKind>().is_err());
    }
}
