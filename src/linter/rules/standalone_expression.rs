use crate::linter::lib::{get_line_column, get_node_text};
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use tree_sitter::Node;

pub struct StandaloneExpressionRule;

impl Rule for StandaloneExpressionRule {
    fn get_target_ast_nodes(&self) -> &[&str] {
        &["expression_statement"]
    }

    fn check_node(&mut self, node: &Node, source_code: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        if let Some(expr_child) = node.child(0) {
            let expr_kind = expr_child.kind();
            if expr_kind != "call"
                && expr_kind != "assignment"
                && expr_kind != "augmented_assignment"
            {
                if matches!(
                    expr_kind,
                    "binary_operator" | "integer" | "float" | "string" | "true" | "false" | "null"
                ) {
                    let (line, column) = get_line_column(&expr_child);
                    let expr_text = get_node_text(&expr_child, source_code);
                    issues.push(LintIssue::new(
                        line,
                        column,
                        "standalone-expression".to_string(),
                        LintSeverity::Warning,
                        format!(
                            "Standalone expression '{}' is not assigned or used, the line may have no effect",
                            expr_text
                        ),
                    ));
                }
            }
        }

        issues
    }
}
