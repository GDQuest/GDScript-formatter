//! This module compares and verifies the structure of the input and output
//! syntax trees (before and after running the formatter).
//!
//! This helps with verifying that the formatter does not drop syntax on very
//! large code files and catching cases where the formatter would break code
//! semantics. It's not a perfect check because the AST does not necessarily
//! capture everything there is to know about the code (the parser may be
//! incomplete; we use this project to test and refine it) and the parser itself
//! may have bugs.
//!
//! Note that the formatter causes some changes to the AST structure that are
//! not semantic changes. So this module has to account for those to avoid false
//! positives. These are:
//!
//! 1. Annotation merge: `@export\nvar x = 1` (annotation sibling + variable
//!    sibling) may become `@export var x = 1` (single variable_statement with
//!    annotations child inside), or vice versa.
//!
//! 2. Splitting statements. E.g., `class_name Foo extends Bar`
//!    (class_name_statement with inline extends child) becomes `class_name
//!    Foo\nextends Bar` (two siblings).
//!
//! 3. Wrapping expressions: the formatter may add parentheses around a long
//!    expression so it can wrap safely. Parenthesized expressions are ignored and
//!    normalized to their inner expression before checking.
//!
//! We normalize ASTs recursively checking for these things before comparing
//! them (i.e. verify that node kind + children match).
use crate::node_kind::GDScriptNodeKind;
use tree_sitter::Node;

/// Lightweight normalized node used for structural comparison.
/// Owns its children so we can compare two normalized trees independently.
#[derive(Debug, Clone, PartialEq, Eq)]
struct NormalizedNode {
    kind: GDScriptNodeKind,
    children: Vec<NormalizedNode>,
}

/// Entry point: compare two tree-sitter trees for structural equivalence,
/// accounting for formatting-induced CST changes.
pub fn trees_structurally_equal(
    input_tree: &tree_sitter::Tree,
    output_tree: &tree_sitter::Tree,
    lookup: &[GDScriptNodeKind; 256],
) -> bool {
    let input = normalize_node(input_tree.root_node(), lookup);
    let output = normalize_node(output_tree.root_node(), lookup);
    input == output
}

fn is_annotatable_declaration(kind: GDScriptNodeKind) -> bool {
    matches!(
        kind,
        GDScriptNodeKind::Variable
            | GDScriptNodeKind::ExportVariable
            | GDScriptNodeKind::OnReadyVariable
            | GDScriptNodeKind::Function
            | GDScriptNodeKind::Constructor
    )
}

fn is_annotation_like(kind: GDScriptNodeKind) -> bool {
    kind == GDScriptNodeKind::Annotation || kind == GDScriptNodeKind::Annotations
}

fn normalize_node(node: Node, lookup: &[GDScriptNodeKind; 256]) -> NormalizedNode {
    let kind = GDScriptNodeKind::get_kind_from_ast_node(node);
    if kind == GDScriptNodeKind::ParenthesizedExpression && node.named_child_count() == 1 {
        let inner = node
            .named_child(0)
            .expect("parenthesized expression with one named child has that child");
        return normalize_node(inner, lookup);
    }
    let canonical_kind = match kind {
        GDScriptNodeKind::ExportVariable | GDScriptNodeKind::OnReadyVariable => {
            GDScriptNodeKind::Variable
        }
        other => other,
    };
    let children = build_normalized_children(node, lookup);
    NormalizedNode {
        kind: canonical_kind,
        children,
    }
}

fn build_normalized_children(node: Node, lookup: &[GDScriptNodeKind; 256]) -> Vec<NormalizedNode> {
    let named_count = node.named_child_count();
    let mut out = Vec::with_capacity(named_count);
    let mut current_child_index: usize = 0;

    while current_child_index < named_count {
        let Some(child) = node.named_child(current_child_index as u32) else {
            current_child_index += 1;
            continue;
        };
        let kind = GDScriptNodeKind::get_kind_from_ast_node(child);

        // Merge consecutive annotations before a declaration. When the formatter
        // moves an annotation from being a sibling to being a child of the
        // declaration (or vice versa), both forms normalize to the same tree.
        if is_annotation_like(kind) {
            let mut annotation_end = current_child_index + 1;
            while annotation_end < named_count {
                let Some(next) = node.named_child(annotation_end as u32) else {
                    break;
                };
                if !is_annotation_like(GDScriptNodeKind::get_kind_from_ast_node(next)) {
                    break;
                }
                annotation_end += 1;
            }
            if annotation_end < named_count {
                let Some(after) = node.named_child(annotation_end as u32) else {
                    current_child_index += 1;
                    continue;
                };
                let after_kind = GDScriptNodeKind::get_kind_from_ast_node(after);
                if is_annotatable_declaration(after_kind) {
                    let mut annotation_children = Vec::new();
                    for annotation_index in current_child_index..annotation_end {
                        let Some(annotation_child) = node.named_child(annotation_index as u32)
                        else {
                            continue;
                        };
                        let annotation_kind =
                            GDScriptNodeKind::get_kind_from_ast_node(annotation_child);
                        if annotation_kind == GDScriptNodeKind::Annotations {
                            let inner_count = annotation_child.named_child_count();
                            for inner_child_index in 0..inner_count {
                                if let Some(inner) =
                                    annotation_child.named_child(inner_child_index as u32)
                                {
                                    annotation_children.push(normalize_node(inner, lookup));
                                }
                            }
                        } else {
                            annotation_children.push(normalize_node(annotation_child, lookup));
                        }
                    }
                    let annotations_wrapper = NormalizedNode {
                        kind: GDScriptNodeKind::Annotations,
                        children: annotation_children,
                    };
                    let declaration_normalized = normalize_node(after, lookup);
                    let mut merged_children =
                        Vec::with_capacity(declaration_normalized.children.len() + 1);
                    merged_children.push(annotations_wrapper);
                    // If the declaration already has an Annotations child as its
                    // first child, skip it (start copying at index 1) since we
                    // just prepended our own annotations wrapper above. Otherwise
                    // copy all declaration children starting at index 0.
                    let declaration_children_copy_start_index: usize =
                        if let Some(first_child) = declaration_normalized.children.first() {
                            if first_child.kind == GDScriptNodeKind::Annotations {
                                1
                            } else {
                                0
                            }
                        } else {
                            0
                        };
                    for child_index in
                        declaration_children_copy_start_index..declaration_normalized.children.len()
                    {
                        merged_children.push(declaration_normalized.children[child_index].clone());
                    }
                    out.push(NormalizedNode {
                        kind: declaration_normalized.kind,
                        children: merged_children,
                    });
                    current_child_index = annotation_end + 1;
                    continue;
                }
            }
            out.push(normalize_node(child, lookup));
            current_child_index += 1;
            continue;
        }

        // Split inline extends from class_name_statement. When reorder is active,
        // class_name Foo extends Bar becomes two separate siblings: class_name Foo
        // followed by extends Bar. Normalize both forms to the same tree.
        if kind == GDScriptNodeKind::ClassName {
            let class_name_child_count = child.named_child_count();
            let mut has_extends = false;
            let mut child_scan_index: usize = 0;
            while child_scan_index < class_name_child_count {
                if let Some(inner) = child.named_child(child_scan_index as u32) {
                    if GDScriptNodeKind::get_kind_from_ast_node(inner) == GDScriptNodeKind::Extends
                    {
                        has_extends = true;
                        break;
                    }
                }
                child_scan_index += 1;
            }
            if has_extends {
                // Build class_children without the extends child.
                let mut class_children: Vec<NormalizedNode> =
                    Vec::with_capacity(class_name_child_count);
                let mut child_scan_index: usize = 0;
                while child_scan_index < class_name_child_count {
                    if let Some(named_child) = child.named_child(child_scan_index as u32) {
                        if GDScriptNodeKind::get_kind_from_ast_node(named_child)
                            != GDScriptNodeKind::Extends
                        {
                            class_children.push(normalize_node(named_child, lookup));
                        }
                    }
                    child_scan_index += 1;
                }
                out.push(NormalizedNode {
                    kind: GDScriptNodeKind::ClassName,
                    children: class_children,
                });
                // Find the extends child and emit it as a separate sibling.
                let mut extends_child: Option<Node> = None;
                let mut child_scan_index: usize = 0;
                while child_scan_index < class_name_child_count {
                    if let Some(named_child) = child.named_child(child_scan_index as u32) {
                        if GDScriptNodeKind::get_kind_from_ast_node(named_child)
                            == GDScriptNodeKind::Extends
                        {
                            extends_child = Some(named_child);
                            break;
                        }
                    }
                    child_scan_index += 1;
                }
                if let Some(extends) = extends_child {
                    out.push(normalize_node(extends, lookup));
                }
                current_child_index += 1;
                continue;
            }
        }

        out.push(normalize_node(child, lookup));
        current_child_index += 1;
    }

    out
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use crate::node_kind::GDScriptNodeKind;

    fn parse(source: &str) -> tree_sitter::Tree {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_gdscript::LANGUAGE.into())
            .unwrap();
        parser.parse(source, None).unwrap()
    }

    fn lookup() -> &'static [GDScriptNodeKind; 256] {
        GDScriptNodeKind::populate_lookup_table()
    }

    fn structurally_equal(a: &str, b: &str) -> bool {
        let ta = parse(a);
        let tb = parse(b);
        trees_structurally_equal(&ta, &tb, lookup())
    }

    #[test]
    fn identical_trees_equal() {
        assert!(structurally_equal("var x = 1", "var x = 1"));
    }

    #[test]
    fn same_structure_equal() {
        assert!(structurally_equal("var x = 1", "var y = 2"));
    }

    #[test]
    fn different_kinds_not_equal() {
        assert!(!structurally_equal("const X = 1", "var x = 1"));
    }

    #[test]
    fn export_single_line_vs_split() {
        assert!(structurally_equal(
            "@export var x = 1",
            "@export\nvar x = 1"
        ));
    }

    #[test]
    fn onready_single_line_vs_split() {
        assert!(structurally_equal(
            "@onready var health := max_health",
            "@onready\nvar health := max_health"
        ));
    }

    #[test]
    fn export_variable_no_newline_stays_merged() {
        assert!(structurally_equal("@export var x = 1", "@export var x = 1"));
    }

    #[test]
    fn class_name_with_inline_extends_normalizes() {
        assert!(structurally_equal(
            "class_name Foo extends Bar",
            "class_name Foo\nextends Bar"
        ));
    }

    #[test]
    fn class_name_without_extends_not_affected() {
        assert!(structurally_equal("class_name Foo", "class_name Foo"));
    }

    #[test]
    fn same_function_shape_equal() {
        assert!(structurally_equal(
            "func foo():\n\tpass",
            "func bar():\n\tpass"
        ));
    }

    #[test]
    fn annotation_before_function_merged() {
        assert!(structurally_equal(
            "@abstract func foo():\n\tpass",
            "@abstract\nfunc foo():\n\tpass"
        ));
    }

    #[test]
    fn complex_source_normalizes() {
        let a = "class_name Foo extends Bar\n@export var x = 1\nfunc test():\n\tpass";
        let b = "class_name Foo\nextends Bar\n@export\nvar x = 1\nfunc test():\n\tpass";
        assert!(structurally_equal(a, b));
    }

    #[test]
    fn export_vs_regular_var_different() {
        assert!(!structurally_equal("@export var x = 1", "var x = 1"));
    }

    #[test]
    fn empty_source_equal() {
        assert!(structurally_equal("", ""));
    }

    #[test]
    fn multiple_annotations_before_var_merge() {
        assert!(structurally_equal(
            "@export @onready var x = 1",
            "@export\n@onready\nvar x = 1"
        ));
    }

    #[test]
    fn if_bare_vs_parenthesized_condition() {
        // When the formatter wraps a long if condition in parentheses for
        // readability, safe mode should treat both forms as equivalent.
        assert!(structurally_equal(
            "if position.x > 200 and position.x < 400:\n\tpass",
            "if (position.x > 200 and position.x < 400):\n\tpass"
        ));
    }

    #[test]
    fn assignment_bare_vs_parenthesized_expression() {
        assert!(structurally_equal(
            "var is_attacking := Input.is_action_pressed(\"attack\") and not animation.is_playing()",
            "var is_attacking := (Input.is_action_pressed(\"attack\") and not animation.is_playing())"
        ));
    }

    #[test]
    fn parentheses_that_change_operator_grouping_are_not_equal() {
        assert!(!structurally_equal(
            "var result = a * (b + c)",
            "var result = a * b + c"
        ));
    }
}
