use crate::linter::lib::{get_node_from_match, get_node_text};
use crate::linter::query_rule::QueryRule;
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use tree_sitter::{Node, Query};

pub struct StandaloneExpressionRule;

impl QueryRule for StandaloneExpressionRule {
    fn query_pattern(&self) -> &'static str {
        r#"(expression_statement) @expr_stmt"#
    }

    fn process_match(
        &self,
        query_match: &tree_sitter::QueryMatch,
        source_code: &str,
        _query: &Query,
    ) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        if let Some(expr_stmt_node) = get_node_from_match(query_match) {
            // Check if it's a standalone expression that's not being used
            if let Some(expr_child) = expr_stmt_node.child(0) {
                let expr_kind = expr_child.kind();
                // Skip function calls and assignments as they have side effects
                if expr_kind != "call"
                    && expr_kind != "assignment"
                    && expr_kind != "augmented_assignment"
                {
                    // Check if it's a simple binary operation or literal
                    if matches!(
                        expr_kind,
                        "binary_operator"
                            | "integer"
                            | "float"
                            | "string"
                            | "true"
                            | "false"
                            | "null"
                    ) {
                        let start_position = expr_child.start_position();
                        let line = start_position.row + 1;
                        let column = start_position.column + 1;

                        let expr_text = get_node_text(&expr_child, source_code);
                        issues.push(LintIssue::new(
                            line,
                            column,
                            "standalone-expression".to_string(),
                            LintSeverity::Warning,
                            format!(
                                "Standalone expression '{}' is not assigned or used, the line may have no effect",
                                expr_text
                            ),
                        ));
                    }
                }
            }
        }

        issues
    }
}

impl Rule for StandaloneExpressionRule {
    fn check(&mut self, source_code: &str, root_node: &Node) -> Result<Vec<LintIssue>, String> {
        QueryRule::check(
            self,
            source_code,
            root_node,
            tree_sitter_gdscript::LANGUAGE.into(),
        )
    }
}
