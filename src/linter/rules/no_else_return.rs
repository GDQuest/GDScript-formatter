use crate::linter::lib::get_line_column;
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use tree_sitter::Node;

pub struct NoElseReturnRule;

impl NoElseReturnRule {
    fn body_ends_with_return(&self, body_node: &Node, _source_code: &str) -> bool {
        let mut cursor = body_node.walk();
        let mut last_statement = None;

        if cursor.goto_first_child() {
            loop {
                let child_node = cursor.node();
                // Skip whitespace and comments
                if !matches!(
                    child_node.kind(),
                    "_newline" | "_indent" | "_dedent" | "comment"
                ) {
                    last_statement = Some(child_node);
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        if let Some(last_stmt) = last_statement {
            return last_stmt.kind() == "return_statement";
        }

        false
    }
}

impl Rule for NoElseReturnRule {
    fn get_target_ast_nodes(&self) -> &[&str] {
        &["if_statement"]
    }

    fn check_node(&mut self, node: &Node, source_code: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        let mut if_body_ends_with_return = false;
        if let Some(body_node) = node.child_by_field_name("body") {
            if_body_ends_with_return = self.body_ends_with_return(&body_node, source_code);
        }

        let mut all_branches_return = if_body_ends_with_return;

        let mut stmt_cursor = node.walk();
        if stmt_cursor.goto_first_child() {
            loop {
                let child_node = stmt_cursor.node();
                if child_node.kind() == "elif_clause" {
                    if if_body_ends_with_return {
                        let (line, column) = get_line_column(&child_node);
                        issues.push(LintIssue::new(
                            line,
                            column,
                            "no-else-return".to_string(),
                            LintSeverity::Warning,
                            "Unnecessary 'elif' after 'if' block that ends with 'return'. Use 'if' instead".to_string(),
                        ));
                    }

                    if let Some(elif_body) = child_node.child_by_field_name("body") {
                        if !self.body_ends_with_return(&elif_body, source_code) {
                            all_branches_return = false;
                        }
                    }
                } else if child_node.kind() == "else_clause" {
                    let (line, column) = get_line_column(&child_node);
                    if all_branches_return {
                        issues.push(LintIssue::new(
                            line,
                            column,
                            "no-else-return".to_string(),
                            LintSeverity::Warning,
                            "Unnecessary 'else' after 'if'/'elif' blocks that end with 'return'"
                                .to_string(),
                        ));
                    }
                }
                if !stmt_cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        issues
    }
}
