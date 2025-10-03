use crate::linter::lib::{get_node_from_match, get_node_text};
use crate::linter::query_rule::QueryRule;
use crate::linter::regex_patterns::{PASCAL_CASE, PRIVATE_SNAKE_CASE, SNAKE_CASE};
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use tree_sitter::{Node, Query};

pub struct VariableNameRule;

impl VariableNameRule {
    fn is_valid_variable_name(&self, name: &str) -> bool {
        SNAKE_CASE.is_match(name) || PRIVATE_SNAKE_CASE.is_match(name)
    }

    fn is_valid_load_variable_name(&self, name: &str) -> bool {
        PASCAL_CASE.is_match(name) || SNAKE_CASE.is_match(name) || PRIVATE_SNAKE_CASE.is_match(name)
    }

    fn is_load_call(&self, node: &Node, source_code: &str) -> bool {
        if node.kind() == "call" {
            if let Some(function_node) = node.child(0) {
                let function_text = function_node
                    .utf8_text(source_code.as_bytes())
                    .unwrap_or("");
                return function_text == "load" || function_text == "preload";
            }
        }
        false
    }
}

impl QueryRule for VariableNameRule {
    fn query_pattern(&self) -> &'static str {
        r#"[
            (variable_statement) @var_stmt
            (export_variable_statement) @var_stmt
            (onready_variable_statement) @var_stmt
        ]"#
    }

    fn process_match(
        &self,
        query_match: &tree_sitter::QueryMatch,
        source_code: &str,
        _query: &Query,
    ) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        // Get the variable statement node
        if let Some(var_stmt_node) = get_node_from_match(query_match) {
            if let Some(name_node) = var_stmt_node.child_by_field_name("name") {
                let name = get_node_text(&name_node, source_code);

                // Check if it's a load variable
                let is_load_var =
                    if let Some(value_node) = var_stmt_node.child_by_field_name("value") {
                        self.is_load_call(&value_node, source_code)
                    } else {
                        false
                    };

                if is_load_var {
                    // For load() variables, only check load rules if they fail normal load validation
                    if !self.is_valid_load_variable_name(name) {
                        let start_position = name_node.start_position();
                        let line = start_position.row + 1;
                        let column = start_position.column + 1;

                        issues.push(LintIssue::new(
                            line,
                            column,
                            "load-variable-name".to_string(),
                            LintSeverity::Error,
                            format!(
                                "Variable name '{}' should be in PascalCase, snake_case or _private_snake_case format",
                                name
                            ),
                        ));
                    }
                    // For preload() variables, do NOT check regular variable rules
                    // PascalCase is allowed for preload variables per the style guide
                } else {
                    // For regular variables, just check regular rules
                    if !self.is_valid_variable_name(name) {
                        let start_position = name_node.start_position();
                        let line = start_position.row + 1;
                        let column = start_position.column + 1;

                        issues.push(LintIssue::new(
                            line,
                            column,
                            "variable-name".to_string(),
                            LintSeverity::Error,
                            format!(
                                "Variable name '{}' should be in snake_case or _private_snake_case format",
                                name
                            ),
                        ));
                    }
                }
            }
        }

        issues
    }
}

impl Rule for VariableNameRule {
    fn check(&mut self, source_code: &str, root_node: &Node) -> Result<Vec<LintIssue>, String> {
        QueryRule::check(
            self,
            source_code,
            root_node,
            tree_sitter_gdscript::LANGUAGE.into(),
        )
    }
}
