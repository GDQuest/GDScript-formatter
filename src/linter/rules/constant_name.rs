use crate::linter::lib::get_node_from_match;
use crate::linter::query_rule::QueryRule;
use crate::linter::regex_patterns::{CONSTANT_CASE, PASCAL_CASE, PRIVATE_CONSTANT_CASE};
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use tree_sitter::{Node, Query};

pub struct ConstantNameRule;

impl ConstantNameRule {
    fn is_valid_constant_name(&self, name: &str) -> bool {
        CONSTANT_CASE.is_match(name) || PRIVATE_CONSTANT_CASE.is_match(name)
    }

    fn is_valid_load_constant_name(&self, name: &str) -> bool {
        // Load constants can use PascalCase or CONSTANT_CASE
        PASCAL_CASE.is_match(name)
            || CONSTANT_CASE.is_match(name)
            || PRIVATE_CONSTANT_CASE.is_match(name)
    }
}

impl QueryRule for ConstantNameRule {
    fn query_pattern(&self) -> &'static str {
        r#"(const_statement) @const_stmt"#
    }

    fn process_match(
        &self,
        query_match: &tree_sitter::QueryMatch,
        source_code: &str,
        _query: &Query,
    ) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        // Get the const_statement node
        if let Some(const_node) = get_node_from_match(query_match) {
            // Extract name and value using field names
            if let (Some(name_node), Some(value_node)) = (
                const_node.child_by_field_name("name"),
                const_node.child_by_field_name("value"),
            ) {
                let name = &source_code[name_node.start_byte()..name_node.end_byte()];
                let value_text = &source_code[value_node.start_byte()..value_node.end_byte()];

                let start_position = name_node.start_position();
                let line = start_position.row + 1;
                let column = start_position.column + 1;

                // Check if the value is a preload call
                let is_preload_const = value_text.trim_start().starts_with("preload(");

                if is_preload_const {
                    // For preload constants, check load naming rules
                    if !self.is_valid_load_constant_name(name) {
                        issues.push(LintIssue::new(
                            line,
                            column,
                            "constant-name".to_string(),
                            LintSeverity::Error,
                            format!(
                                "Preload constant name '{}' should be in PascalCase or CONSTANT_CASE format",
                                name
                            ),
                        ));
                    }
                } else {
                    // For regular constants, check regular naming rules
                    if !self.is_valid_constant_name(name) {
                        issues.push(LintIssue::new(
                            line,
                            column,
                            "constant-name".to_string(),
                            LintSeverity::Error,
                            format!("Constant name '{}' should be in CONSTANT_CASE format", name),
                        ));
                    }
                }
            }
        }

        issues
    }
}

impl Rule for ConstantNameRule {
    fn check(&mut self, source_code: &str, root_node: &Node) -> Result<Vec<LintIssue>, String> {
        QueryRule::check(
            self,
            source_code,
            root_node,
            tree_sitter_gdscript::LANGUAGE.into(),
        )
    }
}
