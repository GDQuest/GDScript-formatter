use crate::linter::lib::{get_node_from_match, get_node_text};
use crate::linter::query_rule::QueryRule;
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use tree_sitter::{Node, Query};

pub struct UnusedArgumentRule;

impl UnusedArgumentRule {
    fn is_identifier_used_in_node(&self, node: &Node, identifier: &str, source_code: &str) -> bool {
        let mut cursor = node.walk();

        fn check_usage(
            cursor: &mut tree_sitter::TreeCursor,
            identifier: &str,
            source_code: &str,
        ) -> bool {
            let node = cursor.node();

            // Check if this is an identifier node that matches our parameter
            if node.kind() == "identifier" {
                let node_text = get_node_text(&node, source_code);
                if node_text == identifier {
                    return true;
                }
            }

            // Recursively check children
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

impl QueryRule for UnusedArgumentRule {
    fn query_pattern(&self) -> &'static str {
        r#"(function_definition) @func"#
    }

    fn process_match(
        &self,
        query_match: &tree_sitter::QueryMatch,
        source_code: &str,
        _query: &Query,
    ) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        // Get the function_definition node
        if let Some(func_node) = get_node_from_match(query_match) {
            let mut parameters = Vec::new();

            // Collect function parameters
            if let Some(params_node) = func_node.child_by_field_name("parameters") {
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

            // Check if parameters are used in function body
            if let Some(body_node) = func_node.child_by_field_name("body") {
                for (param_name, param_node) in parameters {
                    if !self.is_identifier_used_in_node(&body_node, &param_name, source_code) {
                        let start_position = param_node.start_position();
                        let line = start_position.row + 1;
                        let column = start_position.column + 1;

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
        }

        issues
    }
}

impl Rule for UnusedArgumentRule {
    fn check(&mut self, source_code: &str, root_node: &Node) -> Result<Vec<LintIssue>, String> {
        QueryRule::check(
            self,
            source_code,
            root_node,
            tree_sitter_gdscript::LANGUAGE.into(),
        )
    }
}
