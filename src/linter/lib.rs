use tree_sitter::Node;

pub fn get_node_text<'a>(node: &Node, source_code: &'a str) -> &'a str {
    &source_code[node.start_byte()..node.end_byte()]
}

pub fn get_line_column(node: &Node) -> (usize, usize) {
    let start_position = node.start_position();
    (start_position.row + 1, start_position.column + 1)
}
