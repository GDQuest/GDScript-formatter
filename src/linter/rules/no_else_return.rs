use crate::linter::lib::get_line_column;
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use tree_sitter::Node;

pub struct NoElseReturnRule;

impl NoElseReturnRule {
    fn check_no_else_return(&self, node: &Node, source_code: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        let mut cursor = node.walk();

        fn traverse(
            cursor: &mut tree_sitter::TreeCursor,
            rule: &NoElseReturnRule,
            source_code: &str,
            issues: &mut Vec<LintIssue>,
        ) {
            let node = cursor.node();

            if node.kind() == "if_statement" {
                // Check if the if body ends with a return statement
                let mut if_body_ends_with_return = false;
                if let Some(body_node) = node.child_by_field_name("body") {
                    if_body_ends_with_return =
                        rule.body_ends_with_return(&body_node, source_code);
                }

                let mut all_branches_return = if_body_ends_with_return;

                // Check elif and else clauses
                let mut stmt_cursor = node.walk();
                if stmt_cursor.goto_first_child() {
                    loop {
                        let child_node = stmt_cursor.node();
                        if child_node.kind() == "elif_clause" {
                            // If the if block ends with return, elif is unnecessary
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

                            // Check if this elif also ends with return
                            if let Some(elif_body) = child_node.child_by_field_name("body") {
                                if !rule.body_ends_with_return(&elif_body, source_code) {
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
                                    "Unnecessary 'else' after 'if'/'elif' blocks that end with 'return'".to_string(),
                                ));
                            }
                        }
                        if !stmt_cursor.goto_next_sibling() {
                            break;
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
    fn check(&mut self, source_code: &str, root_node: &Node) -> Result<Vec<LintIssue>, String> {
        Ok(self.check_no_else_return(root_node, source_code))
    }
}
