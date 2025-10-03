use crate::linter::lib::{get_line_column, get_node_text};
use crate::linter::regex_patterns::SNAKE_CASE;
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use tree_sitter::Node;

pub struct LoopVariableNameRule;

impl LoopVariableNameRule {
    fn is_valid_loop_variable_name(&self, name: &str) -> bool {
        SNAKE_CASE.is_match(name)
    }

    fn check_loop_variable_names(&self, node: &Node, source_code: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        let mut cursor = node.walk();

        fn traverse(
            cursor: &mut tree_sitter::TreeCursor,
            rule: &LoopVariableNameRule,
            source_code: &str,
            issues: &mut Vec<LintIssue>,
        ) {
            let node = cursor.node();

            if node.kind() == "for_statement" {
                // Look for the loop variable
                // In GDScript, for loops have the pattern: for <variable> in <iterable>:
                // The variable could be an identifier or a typed parameter
                if let Some(left_node) = node.child_by_field_name("left") {
                    let variable_name = if left_node.kind() == "identifier" {
                        get_node_text(&left_node, source_code)
                    } else if left_node.kind() == "typed_parameter" {
                        // For typed loop variables like "for i: int in range(10):"
                        if let Some(name_child) = left_node.child(0) {
                            get_node_text(&name_child, source_code)
                        } else {
                            ""
                        }
                    } else {
                        ""
                    };

                    if !variable_name.is_empty()
                        && !rule.is_valid_loop_variable_name(variable_name)
                    {
                        let (line, column) = get_line_column(&left_node);
                        issues.push(LintIssue::new(
                            line,
                            column,
                            "loop-variable-name".to_string(),
                            LintSeverity::Error,
                            format!(
                                "Loop variable '{}' should be in snake_case format",
                                variable_name
                            ),
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

impl Rule for LoopVariableNameRule {
    fn check(&mut self, source_code: &str, root_node: &Node) -> Result<Vec<LintIssue>, String> {
        Ok(self.check_loop_variable_names(root_node, source_code))
    }
}
