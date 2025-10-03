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

    fn check_constant_names(&self, node: &Node, source_code: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        let mut cursor = node.walk();

        fn traverse(
            cursor: &mut tree_sitter::TreeCursor,
            rule: &ConstantNameRule,
            source_code: &str,
            issues: &mut Vec<LintIssue>,
        ) {
            let node = cursor.node();

            if node.kind() == "const_statement" {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = get_node_text(&name_node, source_code);

                    // Check if it's a load constant
                    let is_preload_const =
                        if let Some(value_node) = node.child_by_field_name("value") {
                            rule.is_preload_call(&value_node, source_code)
                        } else {
                            false
                        };

                    if is_preload_const {
                        // For all load/preload constants, check load naming rules
                        if !rule.is_valid_load_constant_name(&name) {
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

                        // For preload() constants, do NOT check regular constant rules
                        // PascalCase is allowed for preload constants per the style guide
                    } else {
                        // For regular constants, just check regular rules
                        if !rule.is_valid_constant_name(&name) {
                            let (line, column) = get_line_column(&name_node);
                            issues.push(LintIssue::new(
                                line,
                                column,
                                "constant-name".to_string(),
                                LintSeverity::Error,
                                format!(
                                    "Constant name '{}' should be in CONSTANT_CASE format",
                                    name
                                ),
                            ));
                        }
                    }
                }
            }

            if cursor.goto_first_child() {
                loop {
                    traverse(cursor, rule, source_code, issues);
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
                cursor.goto_parent();
            }
        }

        traverse(&mut cursor, self, source_code, &mut issues);
        issues
    }
}

impl Rule for ConstantNameRule {
    fn check(&mut self, source_code: &str, root_node: &Node) -> Result<Vec<LintIssue>, String> {
        Ok(self.check_constant_names(root_node, source_code))
    }
}
