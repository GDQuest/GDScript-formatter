use crate::linter::lib::{get_line_column, get_node_text};
use crate::linter::regex_patterns::{PASCAL_CASE, PRIVATE_SNAKE_CASE, SNAKE_CASE};
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use tree_sitter::Node;
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
                let function_name = get_node_text(&function_node, source_code);
                return function_name == "load" || function_name == "preload";
            }
        }
        false
    }
}

impl Rule for VariableNameRule {
    fn get_target_ast_nodes(&self) -> &[&str] {
        &[
            "variable_statement",
            "export_variable_statement",
            "onready_variable_statement",
        ]
    }

    fn check_node(&mut self, node: &Node, source_code: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        if let Some(name_node) = node.child_by_field_name("name") {
            let name = get_node_text(&name_node, source_code);

            // Check if it's a load variable
            let is_load_var = if let Some(value_node) = node.child_by_field_name("value") {
                self.is_load_call(&value_node, source_code)
            } else {
                false
            };

            if is_load_var {
                // For load() variables, only check load rules if they fail normal load validation
                if !self.is_valid_load_variable_name(name) {
                    let (line, column) = get_line_column(&name_node);
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
            } else {
                // For regular variables, just check regular rules
                if !self.is_valid_variable_name(name) {
                    let (line, column) = get_line_column(&name_node);
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

        issues
    }
}
