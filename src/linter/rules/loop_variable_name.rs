use crate::linter::lib::{get_node_from_match, get_node_text};
use crate::linter::query_rule::QueryRule;
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use tree_sitter::{Node, Query};

pub struct LoopVariableNameRule;

impl QueryRule for LoopVariableNameRule {
    fn query_pattern(&self) -> &'static str {
        r#"(for_statement left: (identifier) @loop_var
          (#not-match? @loop_var "^[a-z_][a-z0-9_]*$"))"#
    }

    fn process_match(
        &self,
        query_match: &tree_sitter::QueryMatch,
        source_code: &str,
        _query: &Query,
    ) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        if let Some(var_node) = get_node_from_match(query_match) {
            let var_name = get_node_text(&var_node, source_code);
            let start_position = var_node.start_position();
            let line = start_position.row + 1;
            let column = start_position.column + 1;

            issues.push(LintIssue::new(
                line,
                column,
                "loop-variable-name".to_string(),
                LintSeverity::Error,
                format!(
                    "Loop variable '{}' should be in snake_case format",
                    var_name
                ),
            ));
        }

        issues
    }
}

impl Rule for LoopVariableNameRule {
    fn check(&mut self, source_code: &str, root_node: &Node) -> Result<Vec<LintIssue>, String> {
        QueryRule::check(
            self,
            source_code,
            root_node,
            tree_sitter_gdscript::LANGUAGE.into(),
        )
    }
}
