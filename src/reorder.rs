//! Compute a `ReorderPlan` that tells the formatter builder which order to
//! visit top-level declarations in. The plan groups declarations by category
//! (signals, enums, consts, vars, funcs, inner classes) and sorts within each
//! category by name, privacy, and method type where applicable.
//!
//! Comments and annotations that precede a declaration in source order are
//! bundled with it so they move together when reordered.

use crate::node_kind::GDScriptNodeKind;
use tree_sitter::Node;

// Public types

#[derive(Debug, Clone)]
pub struct ReorderPlan<'a> {
    pub items: Vec<ReorderItem<'a>>,
}

#[derive(Debug, Clone)]
pub struct ReorderItem<'a> {
    /// Index of this child in the parent node (for direct children).
    pub child_index: usize,
    /// If Some(i), this item refers to the i-th child OF the node at
    /// `child_index`. Used for split inline extends: the extends_statement
    /// is a child of class_name_statement, not a sibling.
    pub sub_child: Option<usize>,
    /// Indices of comment/annotation children that precede this declaration
    /// in source order (they move with it during reorder).
    pub leading_indices: Vec<usize>,
    /// Indices of trailing children (e.g. #endregion) glued to this decl.
    pub trailing_indices: Vec<usize>,
    /// Classification category (determines sort order).
    pub classification: DeclarationKind,
    /// Declaration name for tie-breaking within same category, borrowed from
    /// the source string.
    pub name: &'a str,
    pub is_private: bool,
    pub method_type: Option<MethodType>,
    /// When true, the class_name_statement node contains an inline extends
    /// child that should be skipped when building (emitted as separate item).
    pub split_extends: bool,
}

/// The broad category of a top-level code declaration.
/// This determines the grouping and sort priority of code declarations. This is
/// based on the official style guide.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DeclarationKind {
    ClassAnnotation, // 0: @tool, @icon
    ClassName,       // 1: class_name
    Extends,         // 2: extends
    Docstring,       // 3: ## class doc
    Signal,          // 4
    Enum,            // 5
    Constant,        // 6
    StaticVariable,  // 7
    ExportVariable,  // 8
    RegularVariable, // 9
    OnReadyVariable, // 10
    Method,          // 11: functions (sub-sorted by MethodType)
    InnerClass,      // 12
    Unknown = 255,   // 255
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MethodType {
    StaticInit,         // _static_init()
    StaticFunction,     // static func
    BuiltinVirtual(u8), // _ready, _process, etc. (priority from Godot lifecycle)
    Custom,             // all other user methods
}

/// Builds a `ReorderPlan` for the children of `parent` (typically a `source`
/// node). `content` is the source string used for name extraction.
pub fn build_reorder_plan<'a>(parent: Node<'a>, content: &'a str) -> ReorderPlan<'a> {
    let child_count = parent.child_count();
    let mut items = Vec::with_capacity(child_count);

    // Pass 1: classify each child.
    let mut is_comment = vec![false; child_count];
    let mut is_region_end = vec![false; child_count];

    let mut child_index = 0;
    while child_index < child_count {
        let Some(child) = parent.child(child_index as u32) else {
            child_index += 1;
            continue;
        };
        let kind = GDScriptNodeKind::get_kind_from_ast_node(child);
        if kind == GDScriptNodeKind::Comment
            || kind == GDScriptNodeKind::Annotation
            || kind == GDScriptNodeKind::RegionStart
        {
            is_comment[child_index] = true;
        } else if kind == GDScriptNodeKind::RegionEnd {
            is_comment[child_index] = true;
            is_region_end[child_index] = true;
        } else if kind == GDScriptNodeKind::SemiColon {
            // skip; handled by builder spacing
        } else {
            let child_classification = classify_child(child, content);
            let is_private = child_classification.name.starts_with('_');
            items.push(ReorderItem {
                child_index,
                sub_child: None,
                leading_indices: Vec::new(),
                trailing_indices: Vec::new(),
                classification: child_classification.classification,
                name: child_classification.name,
                is_private,
                method_type: child_classification.method_type,
                split_extends: child_classification.split_extends,
            });

            if child_classification.split_extends {
                if let Some(extends_index) = find_extends_child_index(child) {
                    items.push(ReorderItem {
                        child_index,
                        sub_child: Some(extends_index),
                        leading_indices: Vec::new(),
                        trailing_indices: Vec::new(),
                        classification: DeclarationKind::Extends,
                        name: "",
                        is_private: false,
                        method_type: None,
                        split_extends: false,
                    });
                }
            }
        }
        child_index += 1;
    }

    // Every item pushed so far corresponds to a real declaration (possibly a
    // split-off extends). The docstring item, pushed below, is appended after
    // this point, so this count also bounds pass 2's iteration.
    let declaration_count = items.len();

    // Pass 1b: find class docstring: `##` comments in the header zone
    // (after class_name/extends/annotations, before first signal/enum/etc).
    let mut docstring_indices = Vec::new();
    let mut last_header_child_index: Option<usize> = None;
    let mut last_header_end_byte: Option<usize> = None;
    let mut item_index = 0;
    while item_index < declaration_count {
        let item = &items[item_index];
        if !matches!(
            item.classification,
            DeclarationKind::ClassAnnotation
                | DeclarationKind::ClassName
                | DeclarationKind::Extends
        ) {
            break;
        }
        if let Some(header_child) = parent.child(item.child_index as u32) {
            last_header_child_index = Some(item.child_index);
            last_header_end_byte = Some(header_child.end_byte());
        }
        item_index += 1;
    }
    if let (Some(header_child_index), Some(header_end_byte)) =
        (last_header_child_index, last_header_end_byte)
    {
        let mut previous_end_byte = header_end_byte;
        let mut scan_index = header_child_index + 1;
        while scan_index < child_count {
            let Some(child) = parent.child(scan_index as u32) else {
                scan_index += 1;
                continue;
            };
            if !is_comment[scan_index] || !node_text(child, content).trim_start().starts_with("##")
            {
                break;
            }
            let mut newline_count = 0;
            let mut byte_index = previous_end_byte;
            while byte_index < child.start_byte() {
                if content.as_bytes()[byte_index] == b'\n' {
                    newline_count += 1;
                }
                byte_index += 1;
            }
            // As soon as we found multiple consecutive newlines, it means the
            // next docstring belong to the first declaration in the script so
            // we stop collecting the class-level docstring.
            if newline_count != 1 {
                break;
            }
            docstring_indices.push(scan_index);
            previous_end_byte = child.end_byte();
            scan_index += 1;
        }
    }

    // Mark docstring comments as consumed.
    let mut docstring_index = 0;
    while docstring_index < docstring_indices.len() {
        is_comment[docstring_indices[docstring_index]] = false;
        docstring_index += 1;
    }

    if !docstring_indices.is_empty() {
        items.push(ReorderItem {
            child_index: docstring_indices[0],
            sub_child: None,
            leading_indices: docstring_indices,
            trailing_indices: Vec::new(),
            classification: DeclarationKind::Docstring,
            name: "",
            is_private: false,
            method_type: None,
            split_extends: false,
        });
    }

    // Pass 2: assign leading/trailing children to each declaration.
    let mut previous_declaration_end: Option<usize> = None;
    let mut declaration_index = 0;
    while declaration_index < declaration_count {
        let declaration_child_index = items[declaration_index].child_index;
        let next_declaration_child_index: Option<usize> =
            if declaration_index + 1 < declaration_count {
                Some(items[declaration_index + 1].child_index)
            } else {
                None
            };

        // Leading: all comments (non-region-end) between previous_child decl and this one.
        let start: usize = match previous_declaration_end {
            Some(previous_end) => previous_end + 1,
            None => 0,
        };
        let mut leading = Vec::new();
        let mut scan_index = start;
        while scan_index < declaration_child_index {
            if is_comment[scan_index] && !is_region_end[scan_index] {
                leading.push(scan_index);
            }
            scan_index += 1;
        }
        items[declaration_index].leading_indices = leading;

        // Trailing: region-end between this decl and the next one, or all remaining comments.
        let mut trailing = Vec::new();
        let mut scan_index = declaration_child_index + 1;
        if let Some(next) = next_declaration_child_index {
            while scan_index < next {
                if is_region_end[scan_index] {
                    trailing.push(scan_index);
                }
                scan_index += 1;
            }
        } else {
            // After last declaration: trailing = all remaining comments.
            while scan_index < child_count {
                if is_comment[scan_index] {
                    trailing.push(scan_index);
                }
                scan_index += 1;
            }
        }
        items[declaration_index].trailing_indices = trailing;

        previous_declaration_end = Some(declaration_child_index);
        declaration_index += 1;
    }

    // Pass 3: sort.
    items.sort_by(compare_reorder_items);

    ReorderPlan { items }
}

/// Result of classifying a single child node during reorder planning.
struct ChildClassification<'a> {
    classification: DeclarationKind,
    name: &'a str,
    method_type: Option<MethodType>,
    /// If true, the extends child of a class_name_statement should be split out during reorder.
    split_extends: bool,
}

impl<'a> ChildClassification<'a> {
    /// Builds a classification with no method type and no split extends. This
    /// covers the common case for non-function declarations.
    fn new(classification: DeclarationKind, name: &'a str) -> Self {
        Self {
            classification,
            name,
            method_type: None,
            split_extends: false,
        }
    }
}

fn classify_child<'a>(node: Node<'a>, content: &'a str) -> ChildClassification<'a> {
    let kind = GDScriptNodeKind::get_kind_from_ast_node(node);
    match kind {
        GDScriptNodeKind::Annotation => {
            let name = node_text(node, content);
            ChildClassification::new(DeclarationKind::ClassAnnotation, name)
        }
        GDScriptNodeKind::ClassName => {
            let extends_index = find_extends_child_index(node);
            let name = extract_name(node, content).unwrap_or("unknown_class");
            ChildClassification {
                classification: DeclarationKind::ClassName,
                name,
                method_type: None,
                split_extends: extends_index.is_some(),
            }
        }
        GDScriptNodeKind::Extends => {
            let text = node_text(node, content).trim();
            ChildClassification::new(DeclarationKind::Extends, text)
        }
        GDScriptNodeKind::Signal => {
            let name = extract_name(node, content).unwrap_or("unknown_signal");
            ChildClassification::new(DeclarationKind::Signal, name)
        }
        GDScriptNodeKind::Enum => {
            let name = extract_name(node, content).unwrap_or("unknown_enum");
            ChildClassification::new(DeclarationKind::Enum, name)
        }
        GDScriptNodeKind::Const => {
            let name = extract_name(node, content).unwrap_or("unknown_const");
            ChildClassification::new(DeclarationKind::Constant, name)
        }
        GDScriptNodeKind::Variable => classify_variable(node, content),
        GDScriptNodeKind::ExportVariable => {
            let name = extract_name(node, content).unwrap_or("unknown_var");
            ChildClassification::new(DeclarationKind::ExportVariable, name)
        }
        GDScriptNodeKind::OnReadyVariable => {
            let name = extract_name(node, content).unwrap_or("unknown_var");
            ChildClassification::new(DeclarationKind::OnReadyVariable, name)
        }
        GDScriptNodeKind::Function | GDScriptNodeKind::Constructor => {
            // Constructor nodes have no `name` field. `_init` is a literal keyword.
            let name = if kind == GDScriptNodeKind::Constructor {
                "_init"
            } else {
                extract_name(node, content).unwrap_or("unknown_func")
            };
            let method_type = if name == "_static_init" {
                MethodType::StaticInit
            } else if has_static_keyword_child(node) {
                MethodType::StaticFunction
            } else {
                let priority = get_builtin_virtual_priority(name);
                if priority != 0 {
                    MethodType::BuiltinVirtual(priority)
                } else {
                    MethodType::Custom
                }
            };
            ChildClassification {
                classification: DeclarationKind::Method,
                name,
                method_type: Some(method_type),
                split_extends: false,
            }
        }
        GDScriptNodeKind::ClassDefinition | GDScriptNodeKind::InnerClass => {
            let name = extract_name(node, content).unwrap_or("unknown_class");
            ChildClassification::new(DeclarationKind::InnerClass, name)
        }
        // This ensures the node is preserved during reorder.
        _ => ChildClassification::new(DeclarationKind::Unknown, node_text(node, content)),
    }
}

fn classify_variable<'a>(node: Node<'a>, content: &'a str) -> ChildClassification<'a> {
    let name = extract_name(node, content).unwrap_or("unknown_var");

    if has_annotation_with_name(node, content, "export") {
        ChildClassification::new(DeclarationKind::ExportVariable, name)
    } else if has_annotation_with_name(node, content, "onready") {
        ChildClassification::new(DeclarationKind::OnReadyVariable, name)
    } else if has_static_keyword_child(node) {
        ChildClassification::new(DeclarationKind::StaticVariable, name)
    } else {
        ChildClassification::new(DeclarationKind::RegularVariable, name)
    }
}

/// Extract the "name" field child from a declaration node.
fn extract_name<'a>(node: Node<'a>, content: &'a str) -> Option<&'a str> {
    let count = node.child_count();
    for child_index in 0..count {
        if node.field_name_for_child(child_index as u32) == Some("name") {
            return node
                .child(child_index as u32)
                .map(|child_node| node_text(child_node, content));
        }
    }
    None
}

fn find_extends_child_index(node: Node) -> Option<usize> {
    let count = node.child_count();
    let mut child_index = 0;
    while child_index < count {
        if let Some(child) = node.child(child_index as u32) {
            if GDScriptNodeKind::get_kind_from_ast_node(child) == GDScriptNodeKind::Extends {
                return Some(child_index);
            }
        }
        child_index += 1;
    }
    None
}

/// Returns true if `node` has a direct child with kind `KeywordStatic`.
/// Used to detect `static var` and `static func` declarations by walking
/// the AST instead of matching source text (which is fragile against
/// comments and strings).
fn has_static_keyword_child(node: Node) -> bool {
    let count = node.child_count();
    let mut child_index = 0;
    while child_index < count {
        if let Some(child) = node.child(child_index as u32) {
            if GDScriptNodeKind::get_kind_from_ast_node(child) == GDScriptNodeKind::KeywordStatic {
                return true;
            }
        }
        child_index += 1;
    }
    false
}

/// Returns true if `node` has an `annotations` child containing an
/// `annotation` whose identifier matches `annotation_name`.
///
/// This replaces fragile source-text checks like `text.contains("@export")`
/// which can falsely match comments or strings containing those substrings.
fn has_annotation_with_name(node: Node, content: &str, annotation_name: &str) -> bool {
    let count = node.child_count();
    let mut child_index = 0;
    while child_index < count {
        if let Some(child) = node.child(child_index as u32) {
            if GDScriptNodeKind::get_kind_from_ast_node(child) == GDScriptNodeKind::Annotations
                && annotations_contain_name(child, content, annotation_name)
            {
                return true;
            }
        }
        child_index += 1;
    }
    false
}

/// Walks the children of an `annotations` container node and checks whether
/// any `annotation` child has an `identifier` whose text matches `name`.
fn annotations_contain_name(annotations_node: Node, content: &str, name: &str) -> bool {
    let count = annotations_node.child_count();
    let mut child_index = 0;
    while child_index < count {
        if let Some(child) = annotations_node.child(child_index as u32) {
            if GDScriptNodeKind::get_kind_from_ast_node(child) == GDScriptNodeKind::Annotation
                && annotation_identifier_matches(child, content, name)
            {
                return true;
            }
        }
        child_index += 1;
    }
    false
}

/// Checks whether a single `annotation` node's `identifier` child has text
/// equal to `expected_name`.
fn annotation_identifier_matches(
    annotation_node: Node,
    content: &str,
    expected_name: &str,
) -> bool {
    let count = annotation_node.child_count();
    let mut child_index = 0;
    while child_index < count {
        if let Some(child) = annotation_node.child(child_index as u32) {
            if GDScriptNodeKind::get_kind_from_ast_node(child) == GDScriptNodeKind::Identifier {
                return node_text(child, content) == expected_name;
            }
        }
        child_index += 1;
    }
    false
}

/// Slice the source string at a node's byte range.
/// Tree-sitter byte offsets are always on UTF-8 char boundaries.
fn node_text<'a>(node: Node<'a>, content: &'a str) -> &'a str {
    &content[node.start_byte()..node.end_byte()]
}

/// Maps built-in virtual method names to Godot lifecycle priority.
fn get_builtin_virtual_priority(method_name: &str) -> u8 {
    match method_name {
        "_init" => 1,
        "_enter_tree" => 2,
        "_ready" => 3,
        "_process" => 4,
        "_physics_process" => 5,
        "_exit_tree" => 6,
        "_input" => 7,
        "_unhandled_input" => 8,
        "_unhandled_key_input" | "_gui_input" => 9,
        "_draw" => 10,
        "_notification" => 11,
        "_get_configuration_warnings" => 12,
        "_validate_property" => 13,
        "_get_property_list" => 14,
        "_property_can_revert" => 15,
        "_property_get_revert" => 16,
        "_get" => 17,
        "_set" => 18,
        "_to_string" => 19,
        "_accessibility_get_contextual_info" => 20,
        "_can_drop_data" => 21,
        "_drop_data" => 22,
        "_get_accessibility_container_name" => 23,
        "_get_drag_data" => 24,
        "_get_minimum_size" => 25,
        "_get_tooltip" => 26,
        "_has_point" => 27,
        "_make_custom_tooltip" => 28,
        "_structured_text_parser" => 29,
        _ => 0,
    }
}

fn compare_reorder_items(left: &ReorderItem, right: &ReorderItem) -> std::cmp::Ordering {
    // 1. DeclarationKind (numeric discriminant)
    let kind_cmp = (left.classification as u8).cmp(&(right.classification as u8));
    if kind_cmp != std::cmp::Ordering::Equal {
        return kind_cmp;
    }

    // 2. MethodType sub-sorting for Method items
    if let (Some(method_type_left), Some(method_type_right)) = (left.method_type, right.method_type)
    {
        let type_cmp = method_type_left.cmp(&method_type_right);
        if type_cmp != std::cmp::Ordering::Equal {
            return type_cmp;
        }
    }

    // 3. Privacy: public before pseudo-private
    let privacy_cmp = left.is_private.cmp(&right.is_private);
    if privacy_cmp != std::cmp::Ordering::Equal {
        return privacy_cmp;
    }

    // 4. ClassAnnotation special ordering: @tool < @icon < other
    if left.classification == DeclarationKind::ClassAnnotation
        && right.classification == DeclarationKind::ClassAnnotation
    {
        let priority_left = annotation_priority(left.name);
        let priority_right = annotation_priority(right.name);
        let annotation_cmp = priority_left.cmp(&priority_right);
        if annotation_cmp != std::cmp::Ordering::Equal {
            return annotation_cmp;
        }
    }

    // 5. Stable: original source order (child_index)
    left.child_index.cmp(&right.child_index)
}

fn annotation_priority(text: &str) -> u8 {
    if text.starts_with("@tool") {
        0
    } else if text.starts_with("@icon") {
        1
    } else {
        2
    }
}
