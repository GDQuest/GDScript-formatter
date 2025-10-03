use crate::linter::lib::{get_node_from_match, get_node_text};
use crate::linter::query_rule::QueryRule;
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use tree_sitter::{Node, Query};

pub struct SignalNameRule;

impl QueryRule for SignalNameRule {
    fn query_pattern(&self) -> &'static str {
        r#"(signal_statement (name) @signal_name
          (#not-match? @signal_name "^[a-z][a-z0-9_]*$"))"#
    }

    fn process_match(
        &self,
        query_match: &tree_sitter::QueryMatch,
        source_code: &str,
        _query: &Query,
    ) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        if let Some(name_node) = get_node_from_match(query_match) {
            let name_text = get_node_text(&name_node, source_code);
            let start_position = name_node.start_position();
            let line = start_position.row + 1;
            let column = start_position.column + 1;

            issues.push(LintIssue::new(
                line,
                column,
                "signal-name".to_string(),
                LintSeverity::Error,
                format!("Signal name '{}' should be in snake_case format", name_text),
            ));
        }

        issues
    }
}

impl Rule for SignalNameRule {
    fn check(&mut self, source_code: &str, root_node: &Node) -> Result<Vec<LintIssue>, String> {
        QueryRule::check(
            self,
            source_code,
            root_node,
            tree_sitter_gdscript::LANGUAGE.into(),
        )
    }
}
