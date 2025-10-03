use crate::linter::lib::{get_node_from_match, get_node_text};
use crate::linter::query_rule::QueryRule;
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use tree_sitter::{Node, Query};

pub struct PrivateAccessRule;

impl QueryRule for PrivateAccessRule {
    fn query_pattern(&self) -> &'static str {
        r#"(attribute) @attr"#
    }

    fn process_match(
        &self,
        query_match: &tree_sitter::QueryMatch,
        source_code: &str,
        _query: &Query,
    ) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        // Get the attribute node
        if let Some(attr_node) = get_node_from_match(query_match) {
            // Look for method calls on attributes that start with underscore
            let mut attr_cursor = attr_node.walk();
            if attr_cursor.goto_first_child() {
                // Check if the object is "super" or "self" - if so, allow private method calls
                let object_node = attr_cursor.node();
                let object_text = get_node_text(&object_node, source_code);

                // Skip the object part
                if attr_cursor.goto_next_sibling() && attr_cursor.goto_next_sibling() {
                    let method_node = attr_cursor.node();
                    if method_node.kind() == "attribute_call" {
                        if let Some(method_name_node) = method_node.child(0) {
                            let method_text = method_name_node
                                .utf8_text(source_code.as_bytes())
                                .unwrap_or("");
                            if method_text.starts_with('_')
                                && object_text != "super"
                                && object_text != "self"
                            {
                                let start_position = method_name_node.start_position();
                                let line = start_position.row + 1;
                                let column = start_position.column + 1;

                                issues.push(LintIssue::new(
                                    line,
                                    column,
                                    "private-access".to_string(),
                                    LintSeverity::Error,
                                    format!("Private method '{}' should not be called from outside its class", method_text),
                                ));
                            }
                        }
                    } else if method_node.kind() == "identifier" {
                        let method_text = get_node_text(&method_node, source_code);
                        if method_text.starts_with('_')
                            && object_text != "super"
                            && object_text != "self"
                        {
                            let start_position = method_node.start_position();
                            let line = start_position.row + 1;
                            let column = start_position.column + 1;

                            issues.push(LintIssue::new(
                                line,
                                column,
                                "private-access".to_string(),
                                LintSeverity::Error,
                                format!("Private variable '{}' should not be accessed from outside its class", method_text),
                            ));
                        }
                    }
                }
            }
        }

        issues
    }
}

impl Rule for PrivateAccessRule {
    fn check(&mut self, source_code: &str, root_node: &Node) -> Result<Vec<LintIssue>, String> {
        QueryRule::check(
            self,
            source_code,
            root_node,
            tree_sitter_gdscript::LANGUAGE.into(),
        )
    }
}
