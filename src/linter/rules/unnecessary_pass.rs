use crate::linter::lib::get_node_from_match;
use crate::linter::query_rule::{QueryRule, get_capture_position};
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use tree_sitter::{Node, Query};

pub struct UnnecessaryPassRule;

impl QueryRule for UnnecessaryPassRule {
    fn query_pattern(&self) -> &'static str {
        r#"(body (pass_statement) @pass_stmt)"#
    }

    fn process_match(
        &self,
        query_match: &tree_sitter::QueryMatch,
        _source_code: &str,
        _query: &Query,
    ) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        if let Some(pass_node) = get_node_from_match(query_match) {
            let parent_body = pass_node.parent().unwrap();

            // Check if there are other statements in the body
            let mut has_other_statements = false;
            let mut cursor = parent_body.walk();

            if cursor.goto_first_child() {
                loop {
                    let child = cursor.node();
                    if child.kind() != "pass_statement"
                        && !matches!(child.kind(), "_newline" | "_indent" | "_dedent" | "comment")
                    {
                        has_other_statements = true;
                        break;
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }

            if has_other_statements {
                if let Some((line, column)) = get_capture_position(query_match, 0) {
                    issues.push(LintIssue::new(
                        line,
                        column,
                        "unnecessary-pass".to_string(),
                        LintSeverity::Warning,
                        "Unnecessary 'pass' statement when other statements are present"
                            .to_string(),
                    ));
                }
            }
        }

        issues
    }
}

impl Rule for UnnecessaryPassRule {
    fn check(&mut self, source_code: &str, root_node: &Node) -> Result<Vec<LintIssue>, String> {
        QueryRule::check(
            self,
            source_code,
            root_node,
            tree_sitter_gdscript::LANGUAGE.into(),
        )
    }
}
