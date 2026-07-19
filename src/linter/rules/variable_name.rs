use crate::linter::lib::{get_line_column, get_node_text};
use crate::linter::regex_patterns::{PASCAL_CASE, PRIVATE_SNAKE_CASE, SNAKE_CASE};
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use crate::node_kind::GDScriptNodeKind;
use tree_sitter::Node;
pub struct VariableNameRule;

impl VariableNameRule {
    fn is_valid_variable_name(name: &str) -> bool {
        SNAKE_CASE.is_match(name) || PRIVATE_SNAKE_CASE.is_match(name)
    }

    fn is_valid_load_variable_name(name: &str) -> bool {
        PASCAL_CASE.is_match(name) || SNAKE_CASE.is_match(name) || PRIVATE_SNAKE_CASE.is_match(name)
    }

    fn is_load_call(node: &Node, source_code: &str) -> bool {
        if GDScriptNodeKind::get_kind_from_ast_node(*node) == GDScriptNodeKind::Call
            && let Some(function_node) = node.child(0)
        {
            let function_name = get_node_text(&function_node, source_code);
            return function_name == "load" || function_name == "preload";
        }
        false
    }
}

impl Rule for VariableNameRule {
    fn get_target_ast_nodes(&self) -> &[GDScriptNodeKind] {
        &[
            GDScriptNodeKind::Variable,
            GDScriptNodeKind::ExportVariable,
            GDScriptNodeKind::OnReadyVariable,
        ]
    }

    fn check_node(&mut self, node: &Node, source_code: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        if let Some(name_node) = node.child_by_field_name("name") {
            let name = get_node_text(&name_node, source_code);

            // Check if it's a load variable
            let is_load_var = if let Some(value_node) = node.child_by_field_name("value") {
                Self::is_load_call(&value_node, source_code)
            } else {
                false
            };

            if is_load_var {
                // For load() variables, only check load rules if they fail normal load validation
                if !Self::is_valid_load_variable_name(name) {
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
                if !Self::is_valid_variable_name(name) {
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
