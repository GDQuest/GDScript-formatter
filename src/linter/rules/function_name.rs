use crate::linter::query_rule::{QueryRule, get_capture_position, get_capture_text};
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use tree_sitter::{Node, Query};

pub struct FunctionNameRule;

impl QueryRule for FunctionNameRule {
    fn query_pattern(&self) -> &'static str {
        r#"(function_definition 
            (name) @function_name
            (#not-match? @function_name "^(_)?[a-z][a-z0-9_]*$"))"#
    }

    fn process_match(
        &self,
        query_match: &tree_sitter::QueryMatch,
        source_code: &str,
        _query: &Query,
    ) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        if let (Some(name), Some((line, column))) = (
            get_capture_text(query_match, 0, source_code),
            get_capture_position(query_match, 0),
        ) {
            issues.push(LintIssue::new(
                line,
                column,
                "function-name".to_string(),
                LintSeverity::Error,
                format!(
                    "Function name '{}' should be in snake_case, _private_snake_case format",
                    name
                ),
            ));
        }

        issues
    }
}

impl Rule for FunctionNameRule {
    fn check(&mut self, source_code: &str, root_node: &Node) -> Result<Vec<LintIssue>, String> {
        QueryRule::check(
            self,
            source_code,
            root_node,
            tree_sitter_gdscript::LANGUAGE.into(),
        )
    }
}
