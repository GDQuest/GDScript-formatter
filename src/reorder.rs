//! This module creates a `ReorderPlan` that tells the formatter in which order to
//! visit top-level declarations. The declarations are grouped to apply the
//! sorting within each code region independently.
//!
//! Comments and annotations get attached to relevant declarations so they move
//! together when reordered.
use crate::node_kind::GDScriptNodeKind;
use crate::shared_utils::should_annotation_be_inline;
use tree_sitter::Node;

#[derive(Debug, Clone)]
pub struct ReorderPlan<'a> {
    pub items: Vec<ReorderItem<'a>>,
}

#[derive(Debug, Clone)]
pub struct ReorderItem<'a> {
    /// This index represents one span of the script within which we reorder
    /// declarations. The span is either the inside of a code region or the
    /// outside (before/after the region). We use this index when reordering to
    /// reorder per group.
    group_index: u32,
    /// Index of this child in the parent node.
    pub child_index: u32,
    /// If Some(i), this item refers to the i-th child OF the node at
    /// `child_index`. Used for split inline extends: the extends_statement
    /// is a child of class_name_statement.
    pub sub_child: Option<u32>,
    /// Indices of source children that precede and move with this declaration,
    /// like comments and annotations.
    pub child_indices_attached_before_declaration: Vec<u32>,
    /// Indices of source children that follow and move with this declaration,
    /// like trailing comments.
    pub child_indices_attached_after_declaration: Vec<u32>,
    /// Whether this declaration or its leading comments had a blank line before
    /// it in the source. This preserves blanks lines input by the user in their
    /// source code, up to 1. For example, blank lines used to separate groups
    /// of variables.
    pub has_blank_line_before: bool,
    /// This determines the sort order of this declaration relative to others.
    pub declaration_kind: DeclarationKind,
    /// Declaration name borrowed from the source code. We use this to detect
    /// things like if it's a pseudo-private declaration.
    name: &'a str,
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
    ClassAnnotation, // @tool, @icon
    ClassName,
    Extends,
    ClassDocstring,
    Signal,
    Enum,
    Constant,
    StaticVariable,
    ExportVariable,
    RegularVariable,
    OnReadyVariable,
    Method,
    InnerClass,
    RegionStartOrEnd,
    Unknown = 255,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MethodType {
    StaticInit,         // _static_init()
    StaticFunction,     // static func
    BuiltinVirtual(u8), // _ready, _process, etc.
    Other,
}

impl<'a> ReorderItem<'a> {
    fn new(
        child_index: u32,
        group_index: u32,
        declaration_kind: DeclarationKind,
        name: &'a str,
    ) -> Self {
        Self {
            group_index,
            child_index,
            sub_child: None,
            child_indices_attached_before_declaration: Vec::new(),
            child_indices_attached_after_declaration: Vec::new(),
            has_blank_line_before: false,
            declaration_kind,
            name,
            method_type: None,
            split_extends: false,
        }
    }
}

/// Result of classifying a single child node during reorder planning.
struct ChildClassification<'a> {
    declaration_kind: DeclarationKind,
    name: &'a str,
    method_type: Option<MethodType>,
    /// If true, the extends child of a class_name_statement should be split out during reorder.
    split_extends: bool,
}

impl<'a> ChildClassification<'a> {
    fn new(declaration_kind: DeclarationKind, name: &'a str) -> Self {
        Self {
            declaration_kind,
            name,
            method_type: None,
            split_extends: false,
        }
    }
}

/// Used to track if we found an annotation written on its own line before a
/// declaration.
#[derive(Default)]
struct VisitedAnnotation {
    has_export_annotation: bool,
    has_onready_annotation: bool,
}

/// Slices the source string at a node's byte range and returns a reference to
/// the text.
fn get_node_text<'a>(node: Node<'a>, content: &'a str) -> &'a str {
    &content[node.start_byte()..node.end_byte()]
}

/// Builds a `ReorderPlan` for the children of `parent` (call this on the
/// `source` node, i.e. the root of the AST). `content` is the string of the
/// source code, used to read AST node names.
pub fn build_reorder_plan<'a>(
    gdscript_source_node: Node<'a>,
    source_code: &'a str,
) -> ReorderPlan<'a> {
    let child_count = gdscript_source_node.child_count();
    let mut items = Vec::with_capacity(child_count as usize);

    // Pass 1: classify each child of the parent node.
    //
    // We group items by a group index. The numbers work like this:
    //
    // - Declarations between two region markers share one number, so they
    //   are sorted together and never move across region start and end lines.
    // - Each region start or end line gets its own number, so a marker never
    //   moves either.
    //
    // To get this result we sort the items by their group index.
    let mut current_group_index = 0;

    // We use this to track if we visited a line with an annotation before
    // encountering a declaration.
    //
    // We need something like this with the current code structure because in
    // the AST, annotations can take this form:
    //
    // ```
    // (annotation)
    // (variable_statement)
    // ```
    //
    // And we want to group onready and export variables separately in the class
    // header.
    let mut visited_annotations = VisitedAnnotation::default();

    let mut child_index = 0;
    while child_index < child_count {
        let Some(child) = gdscript_source_node.child(child_index) else {
            child_index += 1;
            continue;
        };
        let kind = GDScriptNodeKind::get_kind_from_ast_node(child);
        if matches!(
            kind,
            GDScriptNodeKind::RegionStart | GDScriptNodeKind::RegionEnd
        ) {
            // We give each region start or end a unique group number so it
            // doesn't get moved anywhere
            current_group_index += 1;
            items.push(ReorderItem::new(
                child_index,
                current_group_index,
                DeclarationKind::RegionStartOrEnd,
                "",
            ));
            current_group_index += 1;
            child_index += 1;
            continue;
        }
        if kind == GDScriptNodeKind::Annotation {
            // Make sure to keep tool and icon at the top of the script, above
            // class_name and extends. Otherwise it's a syntax error.
            let annotation_name = get_node_text(child, source_code);
            if annotation_name.starts_with("@tool") || annotation_name.starts_with("@icon") {
                items.push(ReorderItem::new(
                    child_index,
                    current_group_index,
                    DeclarationKind::ClassAnnotation,
                    annotation_name,
                ));
            } else {
                if let Some(annotation_identifier) = get_annotation_identifier(child, source_code) {
                    if annotation_identifier == "onready" {
                        visited_annotations.has_onready_annotation = true;
                    } else if should_annotation_be_inline(annotation_identifier) {
                        visited_annotations.has_export_annotation = true;
                    }
                }
            }
        } else if matches!(
            kind,
            GDScriptNodeKind::Comment | GDScriptNodeKind::SemiColon
        ) {
            // Skip, we handle those later on
        } else {
            let child_classification = classify_child(child, source_code, &visited_annotations);
            items.push(ReorderItem {
                method_type: child_classification.method_type,
                split_extends: child_classification.split_extends,
                ..ReorderItem::new(
                    child_index,
                    current_group_index,
                    child_classification.declaration_kind,
                    child_classification.name,
                )
            });

            if child_classification.split_extends {
                if let Some(extends_index) =
                    find_child_index_of_kind(child, GDScriptNodeKind::Extends)
                {
                    let mut extends_item = ReorderItem::new(
                        child_index,
                        current_group_index,
                        DeclarationKind::Extends,
                        "",
                    );
                    extends_item.sub_child = Some(extends_index);
                    items.push(extends_item);
                }
            }

            visited_annotations = VisitedAnnotation::default();
        }
        child_index += 1;
    }

    let declaration_count = items.len();

    // Pass 1b: find class docstring after class_name/extends/annotations,
    // before the first signal/enum/etc.
    let mut docstring_indices = Vec::new();
    let mut last_header_child_index: Option<u32> = None;
    let mut last_header_group_index = 0;
    let mut item_index = 0;
    while item_index < declaration_count {
        let item = &items[item_index];
        if item.declaration_kind == DeclarationKind::RegionStartOrEnd {
            item_index += 1;
            continue;
        }
        if !matches!(
            item.declaration_kind,
            DeclarationKind::ClassAnnotation
                | DeclarationKind::ClassName
                | DeclarationKind::Extends
        ) {
            break;
        }
        last_header_child_index = Some(item.child_index);
        last_header_group_index = item.group_index;
        item_index += 1;
    }
    if let Some(header_child_index) = last_header_child_index {
        let header_child = gdscript_source_node
            .child(header_child_index)
            .expect("child index came from this parent");
        let mut previous_end_row = header_child.end_position().row;
        let mut scan_index = header_child_index + 1;
        while scan_index < child_count {
            let Some(child) = gdscript_source_node.child(scan_index) else {
                scan_index += 1;
                continue;
            };
            if GDScriptNodeKind::get_kind_from_ast_node(child) != GDScriptNodeKind::Comment
                || !get_node_text(child, source_code)
                    .trim_start()
                    .starts_with("##")
            {
                break;
            }
            // As soon as we found multiple consecutive newlines, it means the
            // next docstring belong to the first declaration in the script so
            // we stop collecting the class-level docstring.
            if child.start_position().row != previous_end_row + 1 {
                break;
            }
            docstring_indices.push(scan_index);
            previous_end_row = child.end_position().row;
            scan_index += 1;
        }
    }

    let docstring_range = match docstring_indices.first() {
        Some(&start) => start..start + docstring_indices.len() as u32,
        None => 0..0,
    };

    if !docstring_indices.is_empty() {
        let docstring_child_index = docstring_indices[0];
        // The docstring sits directly after the class header in the source,
        // so it belongs to the header's group.
        items.push(ReorderItem {
            child_indices_attached_before_declaration: docstring_indices,
            ..ReorderItem::new(
                docstring_child_index,
                last_header_group_index,
                DeclarationKind::ClassDocstring,
                "",
            )
        });
    }

    // Pass 2: We go through all the AST nodes between declarations and attach
    // them to the preceding or following declaration. This is so that when
    // formatting the output, things like inline comments, doc strings, or
    // annotations stay attached to the correct declaration.
    let mut gap_start = 0;
    for declaration_index in 0..=declaration_count {
        let gap_end = if declaration_index < declaration_count {
            items[declaration_index].child_index
        } else {
            child_count
        };
        let previous_declaration = if declaration_index > 0 {
            gdscript_source_node.child(items[declaration_index - 1].child_index)
        } else {
            None
        };
        for child_index in gap_start..gap_end {
            let child = gdscript_source_node
                .child(child_index)
                .expect("child index came from this parent");
            if docstring_range.contains(&child_index)
                || GDScriptNodeKind::get_kind_from_ast_node(child) == GDScriptNodeKind::SemiColon
            {
                continue;
            }
            if let Some(previous) = previous_declaration
                && (declaration_index == declaration_count
                    || previous.end_position().row == child.start_position().row)
            {
                items[declaration_index - 1]
                    .child_indices_attached_after_declaration
                    .push(child_index);
            } else if declaration_index < declaration_count {
                items[declaration_index]
                    .child_indices_attached_before_declaration
                    .push(child_index);
            }
        }
        if let Some(previous) = previous_declaration
            && declaration_index < declaration_count
        {
            let first_child_index = items[declaration_index]
                .child_indices_attached_before_declaration
                .first()
                .copied()
                .unwrap_or(gap_end);
            let first_child = gdscript_source_node
                .child(first_child_index)
                .expect("child index came from this parent");
            items[declaration_index].has_blank_line_before =
                first_child.start_position().row > previous.end_position().row + 1;
        }
        gap_start = gap_end + 1;
    }

    items.sort_by(compare_reorder_items);
    ReorderPlan { items }
}

fn classify_child<'a>(
    node: Node<'a>,
    content: &'a str,
    visited_annotations: &VisitedAnnotation,
) -> ChildClassification<'a> {
    let kind = GDScriptNodeKind::get_kind_from_ast_node(node);
    match kind {
        GDScriptNodeKind::ClassName => {
            let name = extract_name(node, content).unwrap_or("unknown_class");
            let mut classification = ChildClassification::new(DeclarationKind::ClassName, name);
            classification.split_extends =
                find_child_index_of_kind(node, GDScriptNodeKind::Extends).is_some();
            classification
        }
        GDScriptNodeKind::Extends => {
            let text = get_node_text(node, content).trim();
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
        GDScriptNodeKind::Variable => classify_variable(node, content, visited_annotations),
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
            } else if find_child_index_of_kind(node, GDScriptNodeKind::KeywordStatic).is_some() {
                MethodType::StaticFunction
            } else {
                let priority = get_builtin_virtual_priority(name);
                if priority != 0 {
                    MethodType::BuiltinVirtual(priority)
                } else {
                    MethodType::Other
                }
            };
            let mut classification = ChildClassification::new(DeclarationKind::Method, name);
            classification.method_type = Some(method_type);
            classification
        }
        GDScriptNodeKind::ClassDefinition | GDScriptNodeKind::InnerClass => {
            let name = extract_name(node, content).unwrap_or("unknown_class");
            ChildClassification::new(DeclarationKind::InnerClass, name)
        }
        // This ensures the node is preserved during reorder.
        _ => ChildClassification::new(DeclarationKind::Unknown, get_node_text(node, content)),
    }
}

fn classify_variable<'a>(
    node: Node<'a>,
    content: &'a str,
    visited_annotations: &VisitedAnnotation,
) -> ChildClassification<'a> {
    let name = extract_name(node, content).unwrap_or("unknown_var");

    let mut has_export_annotation = visited_annotations.has_export_annotation;
    let mut has_onready_annotation = visited_annotations.has_onready_annotation;
    for child_index in 0..node.child_count() {
        let Some(child) = node.child(child_index) else {
            continue;
        };
        if GDScriptNodeKind::get_kind_from_ast_node(child) != GDScriptNodeKind::Annotations {
            continue;
        }

        for annotation_index in 0..child.child_count() {
            let Some(annotation) = child.child(annotation_index) else {
                continue;
            };
            if GDScriptNodeKind::get_kind_from_ast_node(annotation) != GDScriptNodeKind::Annotation
            {
                continue;
            }

            let Some(annotation_name) = get_annotation_identifier(annotation, content) else {
                continue;
            };
            if annotation_name == "onready" {
                has_onready_annotation = true;
            } else if should_annotation_be_inline(annotation_name) {
                has_export_annotation = true;
            }
        }
    }

    if has_export_annotation {
        ChildClassification::new(DeclarationKind::ExportVariable, name)
    } else if has_onready_annotation {
        ChildClassification::new(DeclarationKind::OnReadyVariable, name)
    } else if find_child_index_of_kind(node, GDScriptNodeKind::KeywordStatic).is_some() {
        ChildClassification::new(DeclarationKind::StaticVariable, name)
    } else {
        ChildClassification::new(DeclarationKind::RegularVariable, name)
    }
}

/// Returns the identifier name of an annotation node, e.g. `export_range` for
/// `@export_range(0, 100)`.
fn get_annotation_identifier<'a>(annotation: Node<'a>, content: &'a str) -> Option<&'a str> {
    let identifier_index = find_child_index_of_kind(annotation, GDScriptNodeKind::Identifier)?;
    let identifier = annotation.child(identifier_index)?;
    Some(get_node_text(identifier, content))
}

/// Extract the "name" field child from a declaration node.
fn extract_name<'a>(node: Node<'a>, content: &'a str) -> Option<&'a str> {
    let count = node.child_count();
    for child_index in 0..count {
        if node.field_name_for_child(child_index) == Some("name") {
            return node
                .child(child_index)
                .map(|child_node| get_node_text(child_node, content));
        }
    }
    None
}

/// Returns the index of the first direct child of the given node kind, if
/// any. Walking the AST is preferred over matching source text, which is
/// fragile against comments and strings.
fn find_child_index_of_kind(node: Node, kind: GDScriptNodeKind) -> Option<u32> {
    let count = node.child_count();
    let mut child_index = 0;
    while child_index < count {
        if let Some(child) = node.child(child_index) {
            if GDScriptNodeKind::get_kind_from_ast_node(child) == kind {
                return Some(child_index);
            }
        }
        child_index += 1;
    }
    None
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
    // This group index ensures that the code gets reordered by region.
    let ordering_group_index = left.group_index.cmp(&right.group_index);
    if ordering_group_index != std::cmp::Ordering::Equal {
        return ordering_group_index;
    }

    let ordering_declaration_kind =
        (left.declaration_kind as u8).cmp(&(right.declaration_kind as u8));
    if ordering_declaration_kind != std::cmp::Ordering::Equal {
        return ordering_declaration_kind;
    }

    if let (Some(method_type_left), Some(method_type_right)) = (left.method_type, right.method_type)
    {
        let ordering_method_type = method_type_left.cmp(&method_type_right);
        if ordering_method_type != std::cmp::Ordering::Equal {
            return ordering_method_type;
        }
    }

    let ordering_pseudo_private = left.name.starts_with('_').cmp(&right.name.starts_with('_'));
    if ordering_pseudo_private != std::cmp::Ordering::Equal {
        return ordering_pseudo_private;
    }

    // Class annotations have a specific order: @tool < @icon < other potential
    // annotations
    if left.declaration_kind == DeclarationKind::ClassAnnotation
        && right.declaration_kind == DeclarationKind::ClassAnnotation
    {
        fn get_annotation_priority(text: &str) -> u8 {
            match text {
                "@tool" => 0,
                "@icon" => 1,
                _ => 2,
            }
        }

        let priority_left = get_annotation_priority(left.name);
        let priority_right = get_annotation_priority(right.name);
        let ordering_class_annotations = priority_left.cmp(&priority_right);
        if ordering_class_annotations != std::cmp::Ordering::Equal {
            return ordering_class_annotations;
        }
    }

    left.child_index.cmp(&right.child_index)
}
