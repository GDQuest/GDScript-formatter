use crate::linter::query_rule::{QueryRule, get_capture_position, get_capture_text};
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use tree_sitter::{Node, Query};

pub struct EnumNameRule;

impl QueryRule for EnumNameRule {
    fn query_pattern(&self) -> &'static str {
        r#"(enum_definition 
            (name) @enum_name
            (#not-match? @enum_name "^[A-Z][a-zA-Z0-9]*$"))"#
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
                "enum-name".to_string(),
                LintSeverity::Error,
                format!("Enum name '{}' should be in PascalCase format", name),
            ));
        }

        issues
    }
}

impl Rule for EnumNameRule {
    fn check(&mut self, source_code: &str, root_node: &Node) -> Result<Vec<LintIssue>, String> {
        QueryRule::check(
            self,
            source_code,
            root_node,
            tree_sitter_gdscript::LANGUAGE.into(),
        )
    }
}
