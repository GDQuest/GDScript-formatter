use crate::linter::lib::{get_line_column, get_node_text};
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use crate::node_kind::GDScriptNodeKind;
use tree_sitter::Node;

pub struct StandaloneExpressionRule;

impl Rule for StandaloneExpressionRule {
    fn get_target_ast_nodes(&self) -> &[GDScriptNodeKind] {
        &[GDScriptNodeKind::ExpressionStatement]
    }

    fn check_node(&mut self, node: &Node, source_code: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        if let Some(expr_child) = node.child(0) {
            let expr_kind = GDScriptNodeKind::get_kind_from_ast_node(expr_child);
            if expr_kind == GDScriptNodeKind::Call
                || expr_kind == GDScriptNodeKind::Assignment
                || expr_kind == GDScriptNodeKind::AugmentedAssignment
            {
                return issues;
            }

            if matches!(
                expr_kind,
                GDScriptNodeKind::BinaryOperator
                    | GDScriptNodeKind::Literal
                    | GDScriptNodeKind::String
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

        issues
    }
}
