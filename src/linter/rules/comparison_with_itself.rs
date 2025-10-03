use crate::linter::lib::{get_line_column, get_node_text};
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use tree_sitter::Node;

pub struct ComparisonWithItselfRule;

impl ComparisonWithItselfRule {
    fn check_comparison_with_itself(&self, node: &Node, source_code: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        let mut cursor = node.walk();
        fn traverse(
            cursor: &mut tree_sitter::TreeCursor,
            rule: &ComparisonWithItselfRule,
            source_code: &str,
            issues: &mut Vec<LintIssue>,
        ) {
            let node = cursor.node();

            if node.kind() == "binary_operator" {
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
            }

            if cursor.goto_first_child() {
                loop {
                    traverse(cursor, rule, source_code, issues);
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
                cursor.goto_parent();
            }
        }

        traverse(&mut cursor, self, source_code, &mut issues);
        issues
    }
}

impl Rule for ComparisonWithItselfRule {
    fn check(&mut self, source_code: &str, root_node: &Node) -> Result<Vec<LintIssue>, String> {
        Ok(self.check_comparison_with_itself(root_node, source_code))
    }
}
