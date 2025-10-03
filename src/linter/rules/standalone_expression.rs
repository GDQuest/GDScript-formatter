use crate::linter::lib::{get_line_column, get_node_text};
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use tree_sitter::Node;

pub struct StandaloneExpressionRule;

impl StandaloneExpressionRule {
    fn check_standalone_expression(&self, node: &Node, source_code: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        let mut cursor = node.walk();

        fn traverse(
            cursor: &mut tree_sitter::TreeCursor,
            rule: &StandaloneExpressionRule,
            source_code: &str,
            issues: &mut Vec<LintIssue>,
        ) {
            let node = cursor.node();

            if node.kind() == "expression_statement" {
                // Check if it's a standalone expression that's not being used
                if let Some(expr_child) = node.child(0) {
                    let expr_kind = expr_child.kind();
                    // Skip function calls and assignments as they have side effects
                    if expr_kind != "call"
                        && expr_kind != "assignment"
                        && expr_kind != "augmented_assignment"
                    {
                        // Check if it's a simple binary operation or literal
                        if matches!(
                            expr_kind,
                            "binary_operator"
                                | "integer"
                                | "float"
                                | "string"
                                | "true"
                                | "false"
                                | "null"
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

impl Rule for StandaloneExpressionRule {
    fn check(&mut self, source_code: &str, root_node: &Node) -> Result<Vec<LintIssue>, String> {
        Ok(self.check_standalone_expression(root_node, source_code))
    }
}
