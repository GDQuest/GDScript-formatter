use crate::linter::lib::{get_line_column, get_node_text};
use crate::linter::regex_patterns::PASCAL_CASE;
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use tree_sitter::Node;
pub struct EnumNameRule;

impl EnumNameRule {
    fn is_valid_enum_name(&self, name: &str) -> bool {
        PASCAL_CASE.is_match(name)
    }
}

impl Rule for EnumNameRule {
    fn get_target_ast_nodes(&self) -> &[&str] {
        &["enum_definition"]
    }

    fn check_node(&mut self, node: &Node, source_code: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        // Check enum name
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = get_node_text(&name_node, source_code);
            if !self.is_valid_enum_name(name) {
                let (line, column) = get_line_column(&name_node);
                issues.push(LintIssue::new(
                    line,
                    column,
                    "enum-name".to_string(),
                    LintSeverity::Error,
                    format!("Enum name '{}' should be in PascalCase format", name),
                ));
            }
        }

        issues
    }
}
