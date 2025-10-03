use crate::linter::lib::{get_line_column, get_node_text};
use crate::linter::regex_patterns::CONSTANT_CASE;
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use tree_sitter::Node;
pub struct EnumMemberNameRule;

impl EnumMemberNameRule {
    fn is_valid_enum_member_name(&self, name: &str) -> bool {
        CONSTANT_CASE.is_match(name)
    }

    fn check_enum_member_names(&self, node: &Node, source_code: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        let mut cursor = node.walk();

        fn traverse(
            cursor: &mut tree_sitter::TreeCursor,
            rule: &EnumMemberNameRule,
            source_code: &str,
            issues: &mut Vec<LintIssue>,
        ) {
            let node = cursor.node();

            if node.kind() == "enum_definition" {
                // Check enum element names
                if let Some(body_node) = node.child_by_field_name("body") {
                    let mut enum_cursor = body_node.walk();
                    if enum_cursor.goto_first_child() {
                        loop {
                            let enum_member = enum_cursor.node();
                            if enum_member.kind() == "enumerator" {
                                if let Some(element_name_node) =
                                    enum_member.child_by_field_name("left")
                                {
                                    let element_name =
                                        get_node_text(&element_name_node, source_code);
                                    // Skip empty enum member names (happens with empty enums)
                                    if !element_name.is_empty()
                                        && !rule.is_valid_enum_member_name(element_name)
                                    {
                                        let (line, column) = get_line_column(&element_name_node);
                                        issues.push(LintIssue::new(
                                            line,
                                            column,
                                            "enum-member-name".to_string(),
                                            LintSeverity::Error,
                                            format!("Enum element name '{}' should be in CONSTANT_CASE format", element_name),
                                        ));
                                    }
                                }
                            }
                            if !enum_cursor.goto_next_sibling() {
                                break;
                            }
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

impl Rule for EnumMemberNameRule {
    fn check(&mut self, source_code: &str, root_node: &Node) -> Result<Vec<LintIssue>, String> {
        Ok(self.check_enum_member_names(root_node, source_code))
    }
}
