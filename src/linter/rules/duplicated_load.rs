use crate::linter::lib::{get_line_column, get_node_text};
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use std::collections::HashMap;
use tree_sitter::Node;

pub struct DuplicatedLoadRule {
    pub load_paths: HashMap<String, Vec<(usize, usize)>>,
}

impl Rule for DuplicatedLoadRule {
    fn get_target_ast_nodes(&self) -> &[&str] {
        &["call"]
    }

    fn check_node(&mut self, node: &Node, source_code: &str) -> Vec<LintIssue> {
        if let Some(function_node) = node.child(0) {
            let function_name = get_node_text(&function_node, source_code);
            if function_name == "load" || function_name == "preload" {
                if let Some(args_node) = node.child_by_field_name("arguments") {
                    let mut args_cursor = args_node.walk();
                    if args_cursor.goto_first_child() {
                        loop {
                            let arg_node = args_cursor.node();
                            if arg_node.kind() == "string" {
                                let path = get_node_text(&arg_node, source_code);
                                let (line, column) = get_line_column(&arg_node);
                                self.load_paths
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
        Vec::new()
    }

    fn finalize(&mut self, _source_code: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        for (path, locations) in &self.load_paths {
            if locations.len() > 1 {
                for (line, column) in locations {
                    issues.push(LintIssue::new(
                        *line,
                        *column,
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

        self.load_paths.clear();
        issues
    }
}
