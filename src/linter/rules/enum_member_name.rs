use crate::linter::lib::{get_node_from_match, get_node_text};
use crate::linter::query_rule::QueryRule;
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use tree_sitter::{Node, Query};

pub struct EnumMemberNameRule;

impl QueryRule for EnumMemberNameRule {
    fn query_pattern(&self) -> &'static str {
        r#"(enum_definition (enumerator_list (enumerator left: (identifier) @enum_member_name
          (#not-match? @enum_member_name "^[A-Z][A-Z0-9_]*$"))))"#
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

            // Skip empty enum member names (happens with empty enums)
            if !name_text.is_empty() {
                let start_position = name_node.start_position();
                let line = start_position.row + 1;
                let column = start_position.column + 1;

                issues.push(LintIssue::new(
                    line,
                    column,
                    "enum-member-name".to_string(),
                    LintSeverity::Error,
                    format!(
                        "Enum element name '{}' should be in CONSTANT_CASE format",
                        name_text
                    ),
                ));
            }
        }

        issues
    }
}

impl Rule for EnumMemberNameRule {
    fn check(&mut self, source_code: &str, root_node: &Node) -> Result<Vec<LintIssue>, String> {
        QueryRule::check(
            self,
            source_code,
            root_node,
            tree_sitter_gdscript::LANGUAGE.into(),
        )
    }
}
