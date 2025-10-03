use crate::linter::query_rule::{QueryRule, get_capture_position, get_capture_text};
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use std::collections::HashMap;
use tree_sitter::{Node, Query, StreamingIterator};

pub struct DuplicatedLoadRule {
    load_paths: HashMap<String, Vec<(usize, usize)>>,
}

impl DuplicatedLoadRule {
    pub fn new() -> Self {
        Self {
            load_paths: HashMap::new(),
        }
    }
}

impl Default for DuplicatedLoadRule {
    fn default() -> Self {
        Self::new()
    }
}

impl QueryRule for DuplicatedLoadRule {
    fn query_pattern(&self) -> &'static str {
        r#"(call (identifier) @function_name (arguments (string) @path))"#
    }

    fn process_match(
        &self,
        _query_match: &tree_sitter::QueryMatch,
        _source_code: &str,
        _query: &Query,
    ) -> Vec<LintIssue> {
        // We'll collect all matches first, then process duplicates in the Rule implementation
        Vec::new()
    }
}

impl Rule for DuplicatedLoadRule {
    fn check(&mut self, source_code: &str, root_node: &Node) -> Result<Vec<LintIssue>, String> {
        // Reset the load_paths for this check
        self.load_paths.clear();

        // Use tree-sitter queries to find all load/preload calls
        let language = tree_sitter_gdscript::LANGUAGE.into();
        let query = tree_sitter::Query::new(&language, self.query_pattern())
            .map_err(|e| format!("Failed to create query: {:?}", e))?;

        let mut cursor = tree_sitter::QueryCursor::new();
        let mut matches = cursor.matches(&query, *root_node, source_code.as_bytes());

        // Collect all load paths and their positions
        while let Some(query_match) = matches.next() {
            // Check if this is a load or preload function call
            if let Some(function_name) = get_capture_text(&query_match, 0, source_code) {
                if function_name == "load" || function_name == "preload" {
                    if let (Some(path), Some((line, column))) = (
                        get_capture_text(&query_match, 1, source_code), // @path is capture index 1
                        get_capture_position(&query_match, 1),
                    ) {
                        self.load_paths
                            .entry(path.to_string())
                            .or_insert_with(Vec::new)
                            .push((line, column));
                    }
                }
            }
        }

        // Generate issues for duplicated paths
        let mut issues = Vec::new();
        for (path, locations) in &self.load_paths {
            if locations.len() > 1 {
                for &(line, column) in locations {
                    issues.push(LintIssue::new(
                        line,
                        column,
                        "duplicated-load".to_string(),
                        LintSeverity::Warning,
                        format!(
                            "Duplicated load of '{}'. Consider extracting to a constant.",
                            path
                        ),
                    ));
                }
            }
        }

        Ok(issues)
    }
}
