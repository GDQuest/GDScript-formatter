use crate::linter::lib::{get_line_column, get_node_text};
use crate::linter::regex_patterns::{PRIVATE_SNAKE_CASE, SNAKE_CASE};
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use tree_sitter::Node;
pub struct FunctionNameRule;

impl FunctionNameRule {
    fn is_valid_function_name(&self, name: &str) -> bool {
        SNAKE_CASE.is_match(name) || PRIVATE_SNAKE_CASE.is_match(name)
    }
}

impl Rule for FunctionNameRule {
    fn get_target_ast_nodes(&self) -> &[&str] {
        &["function_definition"]
    }

    fn check_node(&mut self, node: &Node, source_code: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        if let Some(name_node) = node.child_by_field_name("name") {
            let name = get_node_text(&name_node, source_code);
            if !self.is_valid_function_name(name) {
                let (line, column) = get_line_column(&name_node);
                issues.push(LintIssue::new(
                    line,
                    column,
                    "function-name".to_string(),
                    LintSeverity::Error,
                    format!(
                        "Function name '{}' should be in snake_case, _private_snake_case format",
                        name
                    ),
                ));
            }
        }

        issues
    }
}
