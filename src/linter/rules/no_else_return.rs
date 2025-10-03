use crate::linter::lib::get_node_from_match;
use crate::linter::query_rule::QueryRule;
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use tree_sitter::{Node, Query};

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

impl QueryRule for NoElseReturnRule {
    fn query_pattern(&self) -> &'static str {
        r#"(if_statement) @if_stmt"#
    }

    fn process_match(
        &self,
        query_match: &tree_sitter::QueryMatch,
        source_code: &str,
        _query: &Query,
    ) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        // Get the if_statement node
        if let Some(if_node) = get_node_from_match(query_match) {
            // Check if the if body ends with a return statement
            let mut if_body_ends_with_return = false;
            if let Some(body_node) = if_node.child_by_field_name("body") {
                if_body_ends_with_return = self.body_ends_with_return(&body_node, source_code);
            }

            let mut all_branches_return = if_body_ends_with_return;

            // Check elif and else clauses
            let mut stmt_cursor = if_node.walk();
            if stmt_cursor.goto_first_child() {
                loop {
                    let child_node = stmt_cursor.node();
                    if child_node.kind() == "elif_clause" {
                        // If the if block ends with return, elif is unnecessary
                        if if_body_ends_with_return {
                            let start_position = child_node.start_position();
                            let line = start_position.row + 1;
                            let column = start_position.column + 1;

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
                            if !self.body_ends_with_return(&elif_body, source_code) {
                                all_branches_return = false;
                            }
                        }
                    } else if child_node.kind() == "else_clause" {
                        if all_branches_return {
                            let start_position = child_node.start_position();
                            let line = start_position.row + 1;
                            let column = start_position.column + 1;

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

        issues
    }
}

impl Rule for NoElseReturnRule {
    fn check(&mut self, source_code: &str, root_node: &Node) -> Result<Vec<LintIssue>, String> {
        QueryRule::check(
            self,
            source_code,
            root_node,
            tree_sitter_gdscript::LANGUAGE.into(),
        )
    }
}
