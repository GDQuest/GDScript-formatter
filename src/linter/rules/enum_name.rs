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

    fn check_enum_names(&self, node: &Node, source_code: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        let mut cursor = node.walk();

        fn traverse(
            cursor: &mut tree_sitter::TreeCursor,
            rule: &EnumNameRule,
            source_code: &str,
            issues: &mut Vec<LintIssue>,
        ) {
            let node = cursor.node();

            if node.kind() == "enum_definition" {
                // Check enum name
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = get_node_text(&name_node, source_code);
                    if !rule.is_valid_enum_name(name) {
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

impl Rule for EnumNameRule {
    fn check(&mut self, source_code: &str, root_node: &Node) -> Result<Vec<LintIssue>, String> {
        Ok(self.check_enum_names(root_node, source_code))
    }
}
