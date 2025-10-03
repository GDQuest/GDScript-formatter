use crate::linter::lib::get_line_column;
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use tree_sitter::Node;
pub struct UnnecessaryPassRule;

impl UnnecessaryPassRule {
    fn check_unnecessary_pass(&self, node: &Node, _source_code: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        let mut cursor = node.walk();
        fn traverse(
            cursor: &mut tree_sitter::TreeCursor,
            rule: &UnnecessaryPassRule,
            issues: &mut Vec<LintIssue>,
        ) {
            let node = cursor.node();

            if node.kind() == "body" || node.kind() == "class_body" {
                let mut has_other_statements = false;
                let mut pass_nodes = Vec::new();

                let mut body_cursor = node.walk();
                if body_cursor.goto_first_child() {
                    loop {
                        let stmt_node = body_cursor.node();
                        if stmt_node.kind() == "pass_statement" {
                            pass_nodes.push(stmt_node);
                        } else if !matches!(
                            stmt_node.kind(),
                            "_newline" | "_indent" | "_dedent" | "comment"
                        ) {
                            has_other_statements = true;
                        }
                        if !body_cursor.goto_next_sibling() {
                            break;
                        }
                    }
                }

                // If there are other statements besides pass, mark pass as unnecessary
                if has_other_statements {
                    for pass_node in pass_nodes {
                        let (line, column) = get_line_column(&pass_node);
                        issues.push(LintIssue::new(
                            line,
                            column,
                            "unnecessary-pass".to_string(),
                            LintSeverity::Warning,
                            "Unnecessary 'pass' statement when other statements are present"
                                .to_string(),
                        ));
                    }
                }
            }

            if cursor.goto_first_child() {
                loop {
                    traverse(cursor, rule, issues);
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
                cursor.goto_parent();
            }
        }

        traverse(&mut cursor, self, &mut issues);
        issues
    }
}

impl Rule for UnnecessaryPassRule {
    fn check(&mut self, source_code: &str, root_node: &Node) -> Result<Vec<LintIssue>, String> {
        Ok(self.check_unnecessary_pass(root_node, source_code))
    }
}
