use crate::linter::lib::{get_line_column, get_node_text};
use crate::linter::regex_patterns::{CONSTANT_CASE, PASCAL_CASE, PRIVATE_CONSTANT_CASE};
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use tree_sitter::Node;

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

    fn is_preload_call(&self, node: &Node, source_code: &str) -> bool {
        if node.kind() == "call" {
            if let Some(function_node) = node.child(0) {
                let function_name = get_node_text(&function_node, source_code);
                return function_name == "preload";
            }
        }

        false
    }
}

impl Rule for ConstantNameRule {
    fn get_target_ast_nodes(&self) -> &[&str] {
        &["const_statement"]
    }

    fn check_node(&mut self, node: &Node, source_code: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        if let Some(name_node) = node.child_by_field_name("name") {
            let name = get_node_text(&name_node, source_code);

            // Check if it's a load constant
            let is_preload_const = if let Some(value_node) = node.child_by_field_name("value") {
                self.is_preload_call(&value_node, source_code)
            } else {
                false
            };

            if is_preload_const {
                // For all load/preload constants, check load naming rules
                if !self.is_valid_load_constant_name(&name) {
                    let (line, column) = get_line_column(&name_node);
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
                // For regular constants, just check regular rules
                if !self.is_valid_constant_name(&name) {
                    let (line, column) = get_line_column(&name_node);
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

        issues
    }
}
