use tree_sitter::{Node, QueryMatch};

pub fn get_node_text<'a>(node: &Node, source_code: &'a str) -> &'a str {
    node.utf8_text(source_code.as_bytes()).unwrap_or("")
}

/// Get the first captured node from a query match by capture index
pub fn get_node_from_match<'a>(query_match: &QueryMatch<'a, 'a>) -> Option<Node<'a>> {
    query_match
        .captures
        .iter()
        .find(|capture| capture.index == 0)
        .map(|capture| capture.node)
}
