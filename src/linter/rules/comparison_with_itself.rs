use crate::linter::lib::{get_line_column, get_node_text};
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use tree_sitter::Node;

pub struct ComparisonWithItselfRule;

impl Rule for ComparisonWithItselfRule {
    fn get_target_ast_nodes(&self) -> &[&str] {
        &["binary_operator"]
    }

    fn check_node(&mut self, node: &Node, source_code: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        if let (Some(left_node), Some(op_node), Some(right_node)) = (
            node.child_by_field_name("left"),
            node.child_by_field_name("op"),
            node.child_by_field_name("right"),
        ) {
            let op = get_node_text(&op_node, source_code);
            if matches!(op, "==" | "!=" | "<" | ">" | "<=" | ">=") {
                let left_text = get_node_text(&left_node, source_code);
                let right_text = get_node_text(&right_node, source_code);

                if left_text == right_text {
                    let (line, column) = get_line_column(&node);
                    issues.push(LintIssue::new(
                        line,
                        column,
                        "comparison-with-itself".to_string(),
                        LintSeverity::Warning,
                        format!(
                            "Redundant comparison '{}' - comparing expression with itself",
                            get_node_text(&node, source_code)
                        ),
                    ));
                }
            }
        }

        issues
    }
}
