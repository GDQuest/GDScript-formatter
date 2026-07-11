use crate::linter::lib::{get_line_column, get_node_text};
use crate::linter::regex_patterns::SNAKE_CASE;
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use crate::node_kind::GDScriptNodeKind;
use tree_sitter::Node;
pub struct SignalNameRule;

impl SignalNameRule {
    fn is_valid_signal_name(name: &str) -> bool {
        SNAKE_CASE.is_match(name)
    }
}

impl Rule for SignalNameRule {
    fn get_target_ast_nodes(&self) -> &[GDScriptNodeKind] {
        &[GDScriptNodeKind::Signal]
    }

    fn check_node(&mut self, node: &Node, source_code: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        if let Some(name_node) = node.child_by_field_name("name") {
            let name = get_node_text(&name_node, source_code);
            if !Self::is_valid_signal_name(name) {
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

        issues
    }
}
