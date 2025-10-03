use crate::linter::lib::{get_node_from_match, get_node_text};
use crate::linter::query_rule::QueryRule;
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use tree_sitter::{Node, Query};

pub struct ClassNameRule;

impl QueryRule for ClassNameRule {
    fn query_pattern(&self) -> &'static str {
        r#"(class_name_statement (name) @class_name
          (#not-match? @class_name "^[A-Z][a-zA-Z0-9]*$"))"#
    }

    fn process_match(
        &self,
        query_match: &tree_sitter::QueryMatch,
        source_code: &str,
        _query: &Query,
    ) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        // Get the class name node that failed the regex match
        if let Some(name_node) = get_node_from_match(query_match) {
            let name_text = get_node_text(&name_node, source_code);
            let start_position = name_node.start_position();
            let line = start_position.row + 1;
            let column = start_position.column + 1;

            issues.push(LintIssue::new(
                line,
                column,
                "class-name".to_string(),
                LintSeverity::Error,
                format!("Class name '{}' should be in PascalCase format", name_text),
            ));
        }

        issues
    }
}

impl Rule for ClassNameRule {
    fn check(&mut self, source_code: &str, root_node: &Node) -> Result<Vec<LintIssue>, String> {
        QueryRule::check(
            self,
            source_code,
            root_node,
            tree_sitter_gdscript::LANGUAGE.into(),
        )
    }
}
