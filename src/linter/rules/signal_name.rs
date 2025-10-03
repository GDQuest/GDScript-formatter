use crate::linter::lib::{get_line_column, get_node_text};
use crate::linter::regex_patterns::SNAKE_CASE;
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use tree_sitter::Node;
pub struct SignalNameRule;

impl SignalNameRule {
    fn is_valid_signal_name(&self, name: &str) -> bool {
        SNAKE_CASE.is_match(name)
    }

    fn check_signal_names(&self, node: &Node, source_code: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        let mut cursor = node.walk();

        fn traverse(
            cursor: &mut tree_sitter::TreeCursor,
            rule: &SignalNameRule,
            source_code: &str,
            issues: &mut Vec<LintIssue>,
        ) {
            let node = cursor.node();

            if node.kind() == "signal_statement" {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = get_node_text(&name_node, source_code);
                    if !rule.is_valid_signal_name(name) {
                        let (line, column) = get_line_column(&name_node);
                        issues.push(LintIssue::new(
                            line,
                            column,
                            "signal-name".to_string(),
                            LintSeverity::Error,
                            format!("Signal name '{}' should be in snake_case format", name),
                        ));
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

impl Rule for SignalNameRule {
    fn check(&mut self, source_code: &str, root_node: &Node) -> Result<Vec<LintIssue>, String> {
        Ok(self.check_signal_names(root_node, source_code))
    }
}
