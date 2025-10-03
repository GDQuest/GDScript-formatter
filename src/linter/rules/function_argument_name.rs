use crate::linter::query_rule::{QueryRule, get_capture_position, get_capture_text};
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use tree_sitter::{Node, Query};

pub struct FunctionArgumentNameRule;

impl QueryRule for FunctionArgumentNameRule {
    fn query_pattern(&self) -> &'static str {
        r#"(function_definition
            (parameters
                [
                    (identifier) @param_name
                    (typed_parameter (identifier) @param_name)
                    (default_parameter (identifier) @param_name)
                    (typed_default_parameter (identifier) @param_name)
                ]
                (#not-match? @param_name "^(_)?[a-z][a-z0-9_]*$")))"#
    }

    fn process_match(
        &self,
        query_match: &tree_sitter::QueryMatch,
        source_code: &str,
        _query: &Query,
    ) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        if let (Some(param_name), Some((line, column))) = (
            get_capture_text(query_match, 0, source_code),
            get_capture_position(query_match, 0),
        ) {
            issues.push(LintIssue::new(
                line,
                column,
                "function-argument-name".to_string(),
                LintSeverity::Error,
                format!(
                    "Function argument '{}' should be in snake_case or _private_snake_case format",
                    param_name
                ),
            ));
        }

        issues
    }
}

impl Rule for FunctionArgumentNameRule {
    fn check(&mut self, source_code: &str, root_node: &Node) -> Result<Vec<LintIssue>, String> {
        QueryRule::check(
            self,
            source_code,
            root_node,
            tree_sitter_gdscript::LANGUAGE.into(),
        )
    }
}
