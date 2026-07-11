use crate::linter::lib::get_line_column;
use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use crate::node_kind::GDScriptNodeKind;
use tree_sitter::Node;
pub struct UnnecessaryPassRule;

impl Rule for UnnecessaryPassRule {
    fn get_target_ast_nodes(&self) -> &[GDScriptNodeKind] {
        &[GDScriptNodeKind::Body, GDScriptNodeKind::ClassBody]
    }

    fn check_node(
        &mut self,
        node: &Node,
        _source_code: &str,
    ) -> Vec<LintIssue> {
        let mut issues = Vec::new();
        let mut has_other_statements = false;
        let mut pass_nodes = Vec::new();

        let mut body_cursor = node.walk();
        if body_cursor.goto_first_child() {
            loop {
                let stmt_node = body_cursor.node();
                if GDScriptNodeKind::get_kind_from_ast_node(stmt_node)
                    == GDScriptNodeKind::PassStatement
                {
                    pass_nodes.push(stmt_node);
                } else {
                    let stmt_kind = GDScriptNodeKind::get_kind_from_ast_node(stmt_node);
                    if stmt_kind != GDScriptNodeKind::Comment
                        && stmt_kind != GDScriptNodeKind::Other
                    {
                        has_other_statements = true;
                    }
                }
                if !body_cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        if has_other_statements {
            for pass_node in pass_nodes {
                let (line, column) = get_line_column(&pass_node);
                issues.push(LintIssue::new(
                    line,
                    column,
                    "unnecessary-pass".to_string(),
                    LintSeverity::Warning,
                    "Unnecessary 'pass' statement when other statements are present".to_string(),
                ));
            }
        }

        issues
    }
}
