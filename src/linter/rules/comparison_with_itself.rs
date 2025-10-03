use crate::linter::query_rule::{get_capture_position, get_capture_text, QueryRule};
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use tree_sitter::{Node, Query};

pub struct ComparisonWithItselfRule;

impl QueryRule for ComparisonWithItselfRule {
    fn query_pattern(&self) -> &'static str {
        r#"(binary_operator
           left: (_) @left
           op: ["==" "!=" "<" ">" "<=" ">="] @op
           right: (_) @right)"#
    }

    fn process_match(
        &self,
        query_match: &tree_sitter::QueryMatch,
        source_code: &str,
        _query: &Query,
    ) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        if let (Some(left_text), Some(op_text), Some(right_text), Some((line, column))) = (
            get_capture_text(query_match, 0, source_code), // @left
            get_capture_text(query_match, 1, source_code), // @op
            get_capture_text(query_match, 2, source_code), // @right
            get_capture_position(query_match, 0), // position of left expression
        ) {
            if left_text == right_text {
                issues.push(LintIssue::new(
                    line,
                    column,
                    "comparison-with-itself".to_string(),
                    LintSeverity::Warning,
                    format!(
                        "Redundant comparison '{}' - comparing expression with itself",
                        format!("{} {} {}", left_text, op_text, right_text)
                    ),
                ));
            }
        }

        issues
    }
}

impl Rule for ComparisonWithItselfRule {
    fn check(&mut self, source_code: &str, root_node: &Node) -> Result<Vec<LintIssue>, String> {
        QueryRule::check(self, source_code, root_node, tree_sitter_gdscript::LANGUAGE.into())
    }
}
