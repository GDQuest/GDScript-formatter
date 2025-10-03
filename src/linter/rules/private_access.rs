use crate::linter::lib::{get_line_column, get_node_text};
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use tree_sitter::Node;
pub struct PrivateAccessRule;

impl Rule for PrivateAccessRule {
    fn get_target_ast_nodes(&self) -> &[&str] {
        &["attribute"]
    }

    fn check_node(&mut self, node: &Node, source_code: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        let mut attr_cursor = node.walk();
        if attr_cursor.goto_first_child() {
            let object_node = attr_cursor.node();
            let object_name = get_node_text(&object_node, source_code);

            if attr_cursor.goto_next_sibling() && attr_cursor.goto_next_sibling() {
                let method_node = attr_cursor.node();
                if method_node.kind() == "attribute_call" {
                    if let Some(method_name_node) = method_node.child(0) {
                        let method_name = get_node_text(&method_name_node, source_code);
                        if method_name.starts_with('_')
                            && object_name != "super"
                            && object_name != "self"
                        {
                            let (line, column) = get_line_column(&method_name_node);
                            issues.push(LintIssue::new(
                                line,
                                column,
                                "private-access".to_string(),
                                LintSeverity::Error,
                                format!("Private method '{}' should not be called from outside its class", method_name),
                            ));
                        }
                    }
                } else if method_node.kind() == "identifier" {
                    let method_name = get_node_text(&method_node, source_code);
                    if method_name.starts_with('_')
                        && object_name != "super"
                        && object_name != "self"
                    {
                        let (line, column) = get_line_column(&method_node);
                        issues.push(LintIssue::new(
                            line,
                            column,
                            "private-access".to_string(),
                            LintSeverity::Error,
                            format!("Private variable '{}' should not be accessed from outside its class", method_name),
                        ));
                    }
                }
            }
        }

        issues
    }
}
