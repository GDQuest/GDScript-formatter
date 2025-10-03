use crate::linter::LintIssue;
use tree_sitter::{Language, Node, Query, QueryCursor, StreamingIterator};

/// A trait for rules that use tree-sitter queries instead of manual tree traversal
pub trait QueryRule {
    /// Returns the tree-sitter query pattern for this rule
    fn query_pattern(&self) -> &'static str;
    
    /// Process a query match and return any lint issues found
    fn process_match(
        &self,
        query_match: &tree_sitter::QueryMatch,
        source_code: &str,
        query: &Query,
    ) -> Vec<LintIssue>;
    
    /// Execute the query-based rule check
    fn check(&self, source_code: &str, root_node: &Node, language: Language) -> Result<Vec<LintIssue>, String> {
        let query = Query::new(&language, self.query_pattern())
            .map_err(|e| format!("Failed to create query: {:?}", e))?;
        
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, *root_node, source_code.as_bytes());
        
        let mut issues = Vec::new();
        while let Some(query_match) = matches.next() {
            issues.extend(self.process_match(&query_match, source_code, &query));
        }
        
        Ok(issues)
    }
}

/// Helper function to get text from a captured node
pub fn get_capture_text<'a>(
    query_match: &tree_sitter::QueryMatch,
    capture_index: u32,
    source_code: &'a str,
) -> Option<&'a str> {
    query_match
        .captures
        .iter()
        .find(|capture| capture.index == capture_index)
        .map(|capture| &source_code[capture.node.start_byte()..capture.node.end_byte()])
}

/// Helper function to get the line and column from a captured node
pub fn get_capture_position(
    query_match: &tree_sitter::QueryMatch,
    capture_index: u32,
) -> Option<(usize, usize)> {
    query_match
        .captures
        .iter()
        .find(|capture| capture.index == capture_index)
        .map(|capture| {
            let start_position = capture.node.start_position();
            (start_position.row + 1, start_position.column + 1)
        })
}

/// Helper function to get a captured node
pub fn get_capture_node<'a>(
    query_match: &'a tree_sitter::QueryMatch<'a, 'a>,
    capture_index: u32,
) -> Option<Node<'a>> {
    query_match
        .captures
        .iter()
        .find(|capture| capture.index == capture_index)
        .map(|capture| capture.node)
}