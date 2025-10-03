use crate::linter::lib::{get_line_column, get_node_text};
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use tree_sitter::Node;

pub struct UnusedArgumentRule;

/// This rule checks for unused function arguments: if a function argument is not used in the function body,
/// it suggests removing it or prefixing it with an underscore (_).
/// Arguments that start with an underscore are ignored by this rule.
impl UnusedArgumentRule {
    fn is_identifier_used_in_node(&self, node: &Node, identifier: &str, source_code: &str) -> bool {
        let mut cursor = node.walk();

        fn check_usage(
            cursor: &mut tree_sitter::TreeCursor,
            identifier: &str,
            source_code: &str,
        ) -> bool {
            let node = cursor.node();

            if node.kind() == "identifier" {
                let node_text = get_node_text(&node, source_code);
                if node_text == identifier {
                    return true;
                }
            }

            if cursor.goto_first_child() {
                loop {
                    if check_usage(cursor, identifier, source_code) {
                        return true;
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
                cursor.goto_parent();
            }

            false
        }

        check_usage(&mut cursor, identifier, source_code)
    }
}

impl Rule for UnusedArgumentRule {
    fn get_target_ast_nodes(&self) -> &[&str] {
        &["function_definition"]
    }

    fn check_node(&mut self, node: &Node, source_code: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();
        let mut parameters = Vec::new();

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

                        if !param_name.is_empty() && !param_name.starts_with('_') {
                            parameters.push((param_name.to_string(), param_node));
                        }
                    }
                    if !params_cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
        }

        if let Some(body_node) = node.child_by_field_name("body") {
            for (param_name, param_node) in parameters {
                if !self.is_identifier_used_in_node(&body_node, &param_name, source_code) {
                    let (line, column) = get_line_column(&param_node);
                    issues.push(LintIssue::new(
                        line,
                        column,
                        "unused-argument".to_string(),
                        LintSeverity::Warning,
                        format!("Function argument '{}' is unused. Consider removing it or prefixing with '_'", param_name),
                    ));
                }
            }
        }

        issues
    }
}
