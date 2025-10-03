use crate::linter::lib::{get_line_column, get_node_text};
use crate::linter::regex_patterns::{PRIVATE_SNAKE_CASE, SNAKE_CASE};
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use tree_sitter::Node;

pub struct FunctionArgumentNameRule;

impl FunctionArgumentNameRule {
    fn is_valid_argument_name(&self, name: &str) -> bool {
        SNAKE_CASE.is_match(name) || PRIVATE_SNAKE_CASE.is_match(name)
    }
}

impl Rule for FunctionArgumentNameRule {
    fn get_target_ast_nodes(&self) -> &[&str] {
        &["function_definition"]
    }

    fn check_node(&mut self, node: &Node, source_code: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        // Check function parameters
        if let Some(params_node) = node.child_by_field_name("parameters") {
            let mut params_cursor = params_node.walk();
            if params_cursor.goto_first_child() {
                loop {
                    let param_node = params_cursor.node();
                    if matches!(
                        param_node.kind(),
                        "identifier"
                            | "typed_parameter"
                            | "default_parameter"
                            | "typed_default_parameter"
                    ) {
                        let param_name = if param_node.kind() == "identifier" {
                            get_node_text(&param_node, source_code)
                        } else if let Some(name_child) = param_node.child(0) {
                            get_node_text(&name_child, source_code)
                        } else {
                            ""
                        };

                        if !param_name.is_empty() && !self.is_valid_argument_name(param_name) {
                            let (line, column) = get_line_column(&param_node);
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
                    }
                    if !params_cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
        }

        issues
    }
}
