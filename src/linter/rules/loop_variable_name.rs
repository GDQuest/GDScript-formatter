use crate::linter::lib::{get_line_column, get_node_text};
use crate::linter::regex_patterns::{PRIVATE_SNAKE_CASE, SNAKE_CASE};
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use crate::node_kind::GDScriptNodeKind;
use tree_sitter::Node;

pub struct LoopVariableNameRule;

impl LoopVariableNameRule {
    fn is_valid_loop_variable_name(name: &str) -> bool {
        SNAKE_CASE.is_match(name) || PRIVATE_SNAKE_CASE.is_match(name)
    }
}

impl Rule for LoopVariableNameRule {
    fn get_target_ast_nodes(&self) -> &[GDScriptNodeKind] {
        &[GDScriptNodeKind::ForStatement]
    }

    fn check_node(
        &mut self,
        node: &Node,
        source_code: &str,
    ) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        // Look for the loop variable
        // In GDScript, for loops have the pattern: for <variable> in <iterable>:
        // The variable could be an identifier or a typed parameter
        if let Some(left_node) = node.child_by_field_name("left") {
            let variable_name = if GDScriptNodeKind::get_kind_from_ast_node(left_node)
                == GDScriptNodeKind::Identifier
            {
                get_node_text(&left_node, source_code)
            } else if GDScriptNodeKind::get_kind_from_ast_node(left_node)
                == GDScriptNodeKind::Parameter
            {
                // For typed loop variables like "for i: int in range(10):"
                if let Some(name_child) = left_node.child(0) {
                    get_node_text(&name_child, source_code)
                } else {
                    ""
                }
            } else {
                ""
            };

            if !variable_name.is_empty() && !Self::is_valid_loop_variable_name(variable_name) {
                let (line, column) = get_line_column(&left_node);
                issues.push(LintIssue::new(
                    line,
                    column,
                    "loop-variable-name".to_string(),
                    LintSeverity::Error,
                    format!(
                        "Loop variable '{}' should be in snake_case_or _private_snake_case format",
                        variable_name
                    ),
                ));
            }
        }

        issues
    }
}
