use crate::linter::lib::{get_line_column, get_node_text};
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use std::collections::HashMap;
use tree_sitter::Node;
pub struct DuplicatedLoadRule;

impl DuplicatedLoadRule {
    fn check_duplicated_load(&self, node: &Node, source_code: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        let mut load_paths: HashMap<String, Vec<(usize, usize)>> = HashMap::new();
        let mut cursor = node.walk();

        fn traverse(
            cursor: &mut tree_sitter::TreeCursor,
            rule: &DuplicatedLoadRule,
            source_code: &str,
            load_paths: &mut HashMap<String, Vec<(usize, usize)>>,
        ) {
            let node = cursor.node();

            if node.kind() == "call" {
                if let Some(function_node) = node.child(0) {
                    let function_name = get_node_text(&function_node, source_code);
                    if function_name == "load" || function_name == "preload" {
                        // Get the arguments
                        if let Some(args_node) = node.child_by_field_name("arguments") {
                            let mut args_cursor = args_node.walk();
                            if args_cursor.goto_first_child() {
                                loop {
                                    let arg_node = args_cursor.node();
                                    if arg_node.kind() == "string" {
                                        let path = get_node_text(&arg_node, source_code);
                                        let (line, column) = get_line_column(&arg_node);
                                        load_paths
                                            .entry(path.to_string())
                                            .or_insert_with(Vec::new)
                                            .push((line, column));
                                    }
                                    if !args_cursor.goto_next_sibling() {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if cursor.goto_first_child() {
                loop {
                    traverse(cursor, rule, source_code, load_paths);
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
                cursor.goto_parent();
            }
        }

        traverse(&mut cursor, self, source_code, &mut load_paths);

        // Check for duplicates
        for (path, locations) in load_paths {
            if locations.len() > 1 {
                for (line, column) in locations {
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

        issues
    }
}

impl Rule for DuplicatedLoadRule {
    fn check(&mut self, source_code: &str, root_node: &Node) -> Result<Vec<LintIssue>, String> {
        Ok(self.check_duplicated_load(root_node, source_code))
    }
}
