//! This module is responsible for formatting the GDScript code in memory. It
//! walks the parsed syntax tree and builds the intermediate representation (IR)
//! that the renderer consumes.
//!
//! For each kind of AST node (arrays, dictionaries, functions, lambdas, etc.)
//! it outputs a flat sequence of `RenderElement` IR nodes: plain text, spaces, line
//! breaks, indent groups, and Wadler groups (see renderer for more info about
//! Wadler's pretty printing algorithm).
//!
//! All formatting decisions (where to put spaces, where the code is allowed to
//! have blank lines, whether a container fits on one line or wraps) live here.

use crate::node_kind::GDScriptNodeKind;
use crate::parser::{ParseInput, RegionWithDisabledFormatting};
use crate::renderer::{RangeRenderElement, RangeSourceBytes, RenderElement};
use crate::reorder::{self, DeclarationKind};

/// Returns the number of bytes a tree-sitter node spans in the source code.
fn node_byte_length(node: tree_sitter::Node) -> usize {
    node.end_byte() - node.start_byte()
}

fn begin_indent(render_elements: &mut Vec<RenderElement>, level: u16) -> usize {
    let index = render_elements.len();
    render_elements.push(RenderElement::Indent {
        level,
        child: RangeRenderElement { start: 0, end: 0 },
    });
    index
}

fn finish_indent(render_elements: &mut [RenderElement], indent_index: usize) {
    let first_child = indent_index + 1;
    let end = render_elements.len();
    if let RenderElement::Indent { child, .. } = &mut render_elements[indent_index] {
        *child = RangeRenderElement {
            start: first_child,
            end,
        };
    }
}

fn begin_group(render_elements: &mut Vec<RenderElement>) -> usize {
    let index = render_elements.len();
    render_elements.push(RenderElement::Group {
        children: RangeRenderElement { start: 0, end: 0 },
    });
    index
}

fn finish_group(render_elements: &mut [RenderElement], group_index: usize) {
    let first_child = group_index + 1;
    let end = render_elements.len();
    if let RenderElement::Group { children, .. } = &mut render_elements[group_index] {
        *children = RangeRenderElement {
            start: first_child,
            end,
        };
    }
}

/// Returns the number of blank lines to output before a declaration of the given
/// kind, based on the user's config. Returns 0 for declarations that don't need
/// extra blank lines (normal spacing applies).
fn get_blank_line_count_before_declaration(input: &ParseInput, kind: GDScriptNodeKind) -> u16 {
    match kind {
        GDScriptNodeKind::Function
        | GDScriptNodeKind::Constructor
        | GDScriptNodeKind::ClassDefinition
        | GDScriptNodeKind::InnerClass => input.blank_lines_around_definitions,
        _ => 0,
    }
}

/// Calculates the blank-line count needed to separate two adjacent
/// declarations.
///
/// Looks at two adjacent declarations and the number of blank lines needed
/// between them, then takes the max of both sides. When the result is zero but
/// `wants_two_blank` is true, it means that neither side directly matches a
/// function/class kind but a comment or annotation leads one that does, we
/// return `blank_lines_around_definitions`.
fn calculate_separator_blank_count(
    input: &ParseInput,
    last_kind: GDScriptNodeKind,
    statement_kind: GDScriptNodeKind,
    wants_two_blank: bool,
) -> u16 {
    let previous_blank_count = get_blank_line_count_before_declaration(input, last_kind);
    let statement_blank_count = get_blank_line_count_before_declaration(input, statement_kind);
    let max_blank_count = previous_blank_count.max(statement_blank_count);
    if max_blank_count == 0 && wants_two_blank {
        input.blank_lines_around_definitions
    } else {
        max_blank_count
    }
}

/// Returns true if the given node kind counts as a "declaration" for the
/// purpose of blank line spacing within a body block (function/class body,
/// etc.): functions, classes, variables, constants, enums, signals, as well
/// as the comments and annotations that lead them.
fn is_declaration(kind: GDScriptNodeKind) -> bool {
    matches!(
        kind,
        GDScriptNodeKind::Function
            | GDScriptNodeKind::Constructor
            | GDScriptNodeKind::ClassDefinition
            | GDScriptNodeKind::Variable
            | GDScriptNodeKind::Const
            | GDScriptNodeKind::Enum
            | GDScriptNodeKind::Signal
            | GDScriptNodeKind::Comment
            | GDScriptNodeKind::Annotation
    )
}

/// Returns true if the given node kind requires two blank lines before it.
fn needs_two_blank_lines(kind: GDScriptNodeKind) -> bool {
    matches!(
        kind,
        GDScriptNodeKind::Function
            | GDScriptNodeKind::ClassDefinition
            | GDScriptNodeKind::InnerClass
            | GDScriptNodeKind::Constructor
    )
}

/// Tracks spacing state that process_source() accumulates as it loops through
/// top-level declarations (vars, funcs, etc. at the root of the script).
struct TopLevelSpacingContext {
    last_output_end: Option<usize>,
    last_declaration_end: Option<usize>,
    last_declaration_kind: Option<GDScriptNodeKind>,
}

/// Scans the source string for newline (`\n`) characters between `from` and
/// `to` (inclusive) and returns the number of newline characters found.
fn count_newlines(source: &str, from: usize, to: usize) -> usize {
    assert!(to >= from);
    let bytes = source.as_bytes();
    let mut count = 0;
    let mut byte_index = from;
    while byte_index < to {
        if bytes[byte_index] == b'\n' {
            count += 1;
        }
        byte_index += 1;
    }
    count
}

fn has_newline(source: &str, from: usize, to: usize) -> bool {
    assert!(to >= from);
    let bytes = source.as_bytes();
    let mut current_index = from;
    while current_index < to {
        if bytes[current_index] == b'\n' {
            return true;
        }
        current_index += 1;
    }
    false
}

fn is_class_header(kind: GDScriptNodeKind) -> bool {
    matches!(
        kind,
        GDScriptNodeKind::ClassName | GDScriptNodeKind::Extends
    )
}

/// Returns true if the node has an `Annotations` child attached to it (and not
/// annotations appearing as siblings before this node).
fn has_own_annotations_child(node: tree_sitter::Node) -> bool {
    let count = node.child_count();
    let mut index = 0;
    while index < count {
        if let Some(child) = node.child(index as u32) {
            if GDScriptNodeKind::get_kind_from_ast_node(child) == GDScriptNodeKind::Annotations {
                return true;
            }
        }
        index += 1;
    }
    false
}

/// Checks the type of an AST node and passes it to the formatter builder
/// function that handles this node kind. This function is called recursively
/// to process all children of the AST node.
fn process_node(
    input: &ParseInput,
    node: tree_sitter::Node,
    render_elements: &mut Vec<RenderElement>,
) {
    let kind = GDScriptNodeKind::get_kind_from_ast_node(node);

    // We reached a leaf AST node after processing all children recursively. We
    // can append the text of this node and return.
    if node.child_count() == 0 {
        let start_byte = node.start_byte();
        let end_byte = node.end_byte();
        if end_byte > start_byte {
            render_elements.push(RenderElement::Text {
                range: RangeSourceBytes {
                    start_byte,
                    end_byte,
                },
            });
        }
        return;
    }

    // These nodes should always be output as-is, without any changes applied
    // (we shouldn't trim whitespace or anything else in strings etc.)
    if matches!(
        kind,
        GDScriptNodeKind::String
            | GDScriptNodeKind::StringName
            | GDScriptNodeKind::NodePath
            | GDScriptNodeKind::GetNode
            | GDScriptNodeKind::RegionStart
            | GDScriptNodeKind::RegionEnd
            | GDScriptNodeKind::Error
    ) {
        render_elements.push(RenderElement::UnformattedSource {
            range: RangeSourceBytes {
                start_byte: node.start_byte(),
                end_byte: node.end_byte(),
            },
        });
        return;
    }

    match kind {
        GDScriptNodeKind::Array
        | GDScriptNodeKind::Dictionary
        | GDScriptNodeKind::EnumeratorList
        | GDScriptNodeKind::Parameters
        | GDScriptNodeKind::Arguments
        | GDScriptNodeKind::SubscriptArguments => process_container(input, node, render_elements),
        GDScriptNodeKind::Body | GDScriptNodeKind::ClassBody | GDScriptNodeKind::MatchBody => {
            process_body(input, node, render_elements)
        }
        GDScriptNodeKind::Lambda => process_lambda(input, node, render_elements),
        GDScriptNodeKind::SetGet => process_setget(input, node, render_elements),
        GDScriptNodeKind::ParenthesizedExpression => {
            process_parenthesized_expression(input, node, render_elements)
        }
        GDScriptNodeKind::BinaryOperator => process_binary_operator(input, node, render_elements),
        GDScriptNodeKind::Condition => process_conditional_expression(input, node, render_elements),
        GDScriptNodeKind::Attribute => process_attribute(input, node, render_elements),
        _ => process_children_with_spacing(input, node, render_elements),
    }
}

/// When the node's child located at start_index onward are annotations, this
/// function scans past all of them to find the next declaration and returns
/// true if it needs two blank lines. We use this so that an annotation leading
/// (i.e. placed right before a declaration) inherits the blank line rules of
/// the declaration it annotates.
fn next_declaration_after_annotations_needs_two_blank_lines(
    node: tree_sitter::Node,
    start_index: usize,
) -> bool {
    let child_count = node.child_count();
    let mut lookahead_index = start_index;
    while lookahead_index < child_count {
        let Some(next_child) = node.child(lookahead_index as u32) else {
            lookahead_index += 1;
            continue;
        };
        let next_kind = GDScriptNodeKind::get_kind_from_ast_node(next_child);
        if next_kind == GDScriptNodeKind::Annotation {
            lookahead_index += 1;
            continue;
        }
        return needs_two_blank_lines(next_kind);
    }
    false
}

/// Returns true when the series of comments starting at start_index is directly above
/// a declaration, which means the comment leads (i.e. documents) that declaration and should
/// inherit the blank line rules of the following declaration.
///
/// For example in this code:
///
/// # This comment explains x
/// func x(): pass
///
/// Here the comment leads function x(), so we use the function's blank line
/// configuration (e.g. 2 blank lines between functions by default).
fn comment_block_leads_declaration(
    source: &str,
    node: tree_sitter::Node,
    start_index: usize,
) -> bool {
    let child_count = node.child_count();
    let mut comment_scan_index = start_index;
    let mut last_comment_end: Option<usize> = None;
    while comment_scan_index < child_count {
        let Some(comment_scan_child) = node.child(comment_scan_index as u32) else {
            comment_scan_index += 1;
            continue;
        };
        let comment_kind = GDScriptNodeKind::get_kind_from_ast_node(comment_scan_child);
        if comment_kind == GDScriptNodeKind::Comment {
            last_comment_end = Some(comment_scan_child.end_byte());
            comment_scan_index += 1;
            continue;
        }
        if let Some(last_comment_end_byte) = last_comment_end {
            if is_declaration(comment_kind) {
                return count_newlines(
                    source,
                    last_comment_end_byte,
                    comment_scan_child.start_byte(),
                ) == 1;
            }
        }
        break;
    }
    false
}

/// Formats a code block or definition's body (i.e. function body, class body,
/// match body). Walks children sequentially and inserts spacing between them:
/// we add blank lines around declarations, annotations stay attached to the
/// nearest declaration, and inline comments force their enclosing group to
/// break.
fn process_body(
    input: &ParseInput,
    node: tree_sitter::Node,
    render_elements: &mut Vec<RenderElement>,
) {
    let source = input.source;
    let indent_index = begin_indent(render_elements, 1);
    let child_count = node.child_count();
    let mut current_index = 0;
    let mut last_processed_child_end_byte: Option<usize> = None;
    let mut last_processed_child_kind: Option<GDScriptNodeKind> = None;
    let mut statement_has_inline_comment = false;

    while current_index < child_count {
        let Some(child) = node.child(current_index as u32) else {
            current_index += 1;
            continue;
        };
        let current_child_kind = GDScriptNodeKind::get_kind_from_ast_node(child);

        // A disabled region's start and end markers can be at different depths
        // in the AST (e.g. the # fmt: off comment can be located at the start
        // of this declaration body, the # fmt: on comment can be located at the
        // end of a sibling node's own if block or for loop body).
        match classify_disabled_region_overlap(input, node, child, current_index) {
            DisabledRegionOverlapKind::CoveredFully(disabled_run) => {
                let region = disabled_run.region;
                if child.start_byte() == region.start {
                    if let Some(previous_end) = last_processed_child_end_byte {
                        // region.start points at the off marker's # character
                        // so we can count the number of newlines between the
                        // previous child and the off marker using the region
                        // start.
                        let newline_count = count_newlines(source, previous_end, region.start);
                        push_separator_for_newline_count(newline_count, render_elements);
                    }
                    render_elements.push(RenderElement::UnformattedSource {
                        range: RangeSourceBytes {
                            start_byte: region.start,
                            end_byte: region.end,
                        },
                    });
                }
                let last_covered_child = node
                    .child(disabled_run.last_covered_index as u32)
                    .expect("last_covered_index came from this same node's children");
                last_processed_child_end_byte = Some(last_covered_child.end_byte());
                last_processed_child_kind =
                    Some(GDScriptNodeKind::get_kind_from_ast_node(last_covered_child));
                statement_has_inline_comment = false;
                current_index = disabled_run.last_covered_index + 1;
                continue;
            }
            DisabledRegionOverlapKind::PartiallyCovered => {
                // the child's first bytes are already part of a parent region's
                // text; skip adding a separator (it would duplicate whitespace)
                // and process the child directly. process_node() will process
                // children and when it returns we will resume from this child's
                // end byte.
                process_node(input, child, render_elements);
                last_processed_child_end_byte = Some(child.end_byte());
                last_processed_child_kind = Some(current_child_kind);
                statement_has_inline_comment = false;
                current_index += 1;
                continue;
            }
            DisabledRegionOverlapKind::None => {}
        }

        if current_child_kind == GDScriptNodeKind::SemiColon {
            render_elements.push(RenderElement::HardLine);
            last_processed_child_end_byte = Some(child.end_byte());
            last_processed_child_kind = Some(GDScriptNodeKind::SemiColon);
            current_index += 1;
            continue;
        }
        if let Some(previous_end) = last_processed_child_end_byte {
            if last_processed_child_kind == Some(GDScriptNodeKind::SemiColon) {
                let newline_count = count_newlines(source, previous_end, child.start_byte());
                if newline_count >= 2 {
                    render_elements.push(RenderElement::BlankLine);
                    render_elements.push(RenderElement::BlankLine);
                }
            } else {
                let current_is_declaration = is_declaration(current_child_kind);
                let previous_is_declaration =
                    is_declaration(last_processed_child_kind.unwrap_or(GDScriptNodeKind::Other));
                if last_processed_child_kind == Some(GDScriptNodeKind::Annotation) {
                    if current_child_kind != GDScriptNodeKind::Annotation
                        && current_child_kind != GDScriptNodeKind::Comment
                        && current_is_declaration
                        && !has_own_annotations_child(child)
                    {
                        render_elements.push(RenderElement::Space);
                    } else if has_newline(source, previous_end, child.start_byte()) {
                        render_elements.push(RenderElement::HardLine);
                    } else {
                        render_elements.push(RenderElement::Space);
                    }
                } else if last_processed_child_kind == Some(GDScriptNodeKind::Comment) {
                    if statement_has_inline_comment && current_is_declaration {
                        let needs_two_blank_lines = needs_two_blank_lines(current_child_kind);
                        add_spacing_between_body_children(
                            previous_end,
                            child.start_byte(),
                            input,
                            render_elements,
                            last_processed_child_kind,
                            current_child_kind,
                            needs_two_blank_lines,
                        );
                    } else {
                        render_elements.push(RenderElement::HardLine);
                    }
                } else if previous_is_declaration && current_is_declaration {
                    let current_is_annotation = current_child_kind == GDScriptNodeKind::Annotation;
                    let current_target_needs_two_blank_lines = if current_is_annotation {
                        next_declaration_after_annotations_needs_two_blank_lines(
                            node,
                            current_index + 1,
                        )
                    } else if current_child_kind == GDScriptNodeKind::Comment {
                        comment_block_leads_declaration(source, node, current_index)
                    } else {
                        needs_two_blank_lines(current_child_kind)
                    };
                    let needs_two_blank_lines = if GDScriptNodeKind::get_kind_from_ast_node(node)
                        == GDScriptNodeKind::ClassBody
                    {
                        let previous_needs_two_blank = needs_two_blank_lines(
                            last_processed_child_kind.unwrap_or(GDScriptNodeKind::Other),
                        );
                        previous_needs_two_blank || current_target_needs_two_blank_lines
                    } else {
                        current_target_needs_two_blank_lines
                    };
                    add_spacing_between_body_children(
                        previous_end,
                        child.start_byte(),
                        input,
                        render_elements,
                        last_processed_child_kind,
                        current_child_kind,
                        needs_two_blank_lines,
                    );
                } else {
                    let newline_count = count_newlines(source, previous_end, child.start_byte());
                    push_separator_for_newline_count(newline_count, render_elements);
                }
            }
        }

        process_node(input, child, render_elements);
        if current_child_kind == GDScriptNodeKind::Comment {
            // Check if this comment sits on the same line as the previous child.
            // To do so, we check if there's a newline between the end of the previous
            // child and the start of this comment.
            if let Some(previous_child_end_byte) = last_processed_child_end_byte {
                statement_has_inline_comment =
                    !has_newline(source, previous_child_end_byte, child.start_byte());
            }
        } else {
            statement_has_inline_comment = false;
        }
        last_processed_child_end_byte = Some(child.end_byte());
        last_processed_child_kind = Some(current_child_kind);
        current_index += 1;
    }
    finish_indent(render_elements, indent_index);
}
/// output N blank lines. N=0 outputs nothing, N=1 outputs a single BlankLine, N>=2
/// outputs N BlankLines.
fn push_blank_lines(render_elements: &mut Vec<RenderElement>, count: u16) {
    let mut remaining = count;
    while remaining > 0 {
        render_elements.push(RenderElement::BlankLine);
        remaining -= 1;
    }
}

/// Pushes the separator we need between two sibling nodes based on the number
/// of newlines found between them in the source: 0 newlines keeps them on the
/// same line (a space), 1 newline is a normal line break, and 2+ newlines
/// becomes a a hard line followed by a blank line.
fn push_separator_for_newline_count(
    newline_count: usize,
    render_elements: &mut Vec<RenderElement>,
) {
    if newline_count == 0 {
        render_elements.push(RenderElement::Space);
    } else if newline_count == 1 {
        render_elements.push(RenderElement::HardLine);
    } else {
        render_elements.push(RenderElement::HardLine);
        render_elements.push(RenderElement::BlankLine);
    }
}

/// Represents a sequence of sibling tree-sitter AST nodes covered by one region
/// with disabled formatting. The region's byte range, and the index of the last
/// sibling AST node covered by it. Every sibling in this span gets output as a
/// raw string .
struct DisabledRegionNodeSpan {
    region: RegionWithDisabledFormatting,
    /// The index of the last sibling AST node covered by this region.
    last_covered_index: usize,
}

/// If byte_offset falls inside one of the input's disabled regions, returns
/// that region.
fn find_disabled_region_containing(
    input: &ParseInput,
    byte_offset: usize,
) -> Option<RegionWithDisabledFormatting> {
    for region in &input.disabled_regions {
        if region.start > byte_offset {
            break;
        }
        if byte_offset < region.end {
            return Some(*region);
        }
    }
    None
}

/// Describes the three ways a child can relate to a disabled region, as seen by whichever
/// loop (process_source(), process_body(), process_children_with_spacing()) is
/// currently iterating its parent's children.
///
/// A disabled formatting region's start and end comments can be at different
/// depths in the tree sitter AST (e.g. the off marker before a function, the on
/// marker inside that function's body). A challenge is that as we go through
/// the tree recursively, the closing marker for the region might be in the
/// middle of a parent AST node. Example:
///
/// # fmt: off
/// func my_func():
///     var my_var  =  "hi"
///     # fmt: on
///     print(  my_var)
///
/// Here the disabling region overlaps the function node and the body node, but
/// does not cover them fully. It should leave the function definition and my_var
/// unformatted but remove the spaces in the print() function call.
enum DisabledRegionOverlapKind {
    /// The node is fully covered by a disabled region. It's safe to skip
    /// checking its children entirely (or, if it is this region's off marker,
    /// to close the region's UnformattedSource).
    CoveredFully(DisabledRegionNodeSpan),
    /// A disabled region starts before the node and ends somewhere inside
    /// the node's own subtree. We need to dive into the node's children
    /// to determine how to handle it.
    PartiallyCovered,
    /// The node is not affected by any disabled region.
    None,
}

/// Classifies how first_child relates to the input's disabled regions.
fn classify_disabled_region_overlap(
    input: &ParseInput,
    node: tree_sitter::Node,
    first_child: tree_sitter::Node,
    first_child_index: usize,
) -> DisabledRegionOverlapKind {
    let Some(region) = find_disabled_region_containing(input, first_child.start_byte()) else {
        return DisabledRegionOverlapKind::None;
    };

    // A region can end deeper than this level, e.g. an # fmt: off comment that
    // starts before a function and a matching # fmt: on sitting inside that
    // function's body. When first_child only partially overlaps the region this
    // way, there's a partial overlap and we'll have to process its children
    if first_child.start_byte() != region.start && first_child.end_byte() > region.end {
        return DisabledRegionOverlapKind::PartiallyCovered;
    }

    let child_count = node.child_count();
    let mut last_covered_index = first_child_index;
    let mut scan_index = first_child_index + 1;
    while scan_index < child_count {
        let Some(next_child) = node.child(scan_index as u32) else {
            scan_index += 1;
            continue;
        };
        if next_child.start_byte() >= region.end {
            break;
        }
        if next_child.end_byte() > region.end {
            break;
        }
        last_covered_index = scan_index;
        scan_index += 1;
    }
    DisabledRegionOverlapKind::CoveredFully(DisabledRegionNodeSpan {
        region,
        last_covered_index,
    })
}

/// Starts formatting code from the source node (which is the topmost
/// tree-sitter AST node).
fn process_source(
    input: &ParseInput,
    node: tree_sitter::Node,
    render_elements: &mut Vec<RenderElement>,
) {
    // Reordering moves declarations around based on their kind, which would
    // pull code into or out of a # fmt: off disabled region. For now we
    // skip reordering for disabled regions, but in the future we may want
    // to reorder code around disabled regions as well?
    if input.reorder_code {
        if input.disabled_regions.is_empty() {
            process_source_reorder(input, node, render_elements);
            return;
        }
        println!(
            "The code uses disabled regions. Reordering is currently incompatible with disabled formatting as it can span any lines and reordering may break the disabled regions. Skipping reordering."
        );
    }
    let source = input.source;
    let child_count = node.child_count();
    let mut current_index = 0;
    let mut spacing_context = TopLevelSpacingContext {
        last_output_end: None,
        last_declaration_end: None,
        last_declaration_kind: None,
    };
    // Pending leading block: comments and standalone annotations buffered
    // before the next declaration. Each entry carries the number of source
    // newlines in the gap immediately before it (relative to the previous
    // output or buffered item).
    let mut pending: Vec<(tree_sitter::Node, usize)> = Vec::new();

    while current_index < child_count {
        let Some(child) = node.child(current_index as u32) else {
            current_index += 1;
            continue;
        };

        let kind = GDScriptNodeKind::get_kind_from_ast_node(child);

        // This code is similar to the one in process_body(). See comments
        // there for some explanation of what this does and why it's needed.
        match classify_disabled_region_overlap(input, node, child, current_index) {
            DisabledRegionOverlapKind::CoveredFully(disabled_run) => {
                let region = disabled_run.region;
                if child.start_byte() == region.start {
                    output_pending_before_declaration(
                        render_elements,
                        input,
                        &mut pending,
                        &spacing_context,
                        child,
                    );
                    render_elements.push(RenderElement::UnformattedSource {
                        range: RangeSourceBytes {
                            start_byte: region.start,
                            end_byte: region.end,
                        },
                    });
                }
                let last_covered_child = node
                    .child(disabled_run.last_covered_index as u32)
                    .expect("last_covered_index came from this same node's children");
                spacing_context.last_output_end = Some(last_covered_child.end_byte());
                spacing_context.last_declaration_end = Some(last_covered_child.end_byte());
                spacing_context.last_declaration_kind =
                    Some(GDScriptNodeKind::get_kind_from_ast_node(last_covered_child));
                current_index = disabled_run.last_covered_index + 1;
                continue;
            }
            DisabledRegionOverlapKind::PartiallyCovered => {
                process_node(input, child, render_elements);
                spacing_context.last_output_end = Some(child.end_byte());
                spacing_context.last_declaration_end = Some(child.end_byte());
                spacing_context.last_declaration_kind = Some(kind);
                current_index += 1;
                continue;
            }
            DisabledRegionOverlapKind::None => {}
        }

        if kind == GDScriptNodeKind::SemiColon {
            // We want to remove semicolons in the formatter. When we encounter
            // one that separates declarations, we put next declarations on a new
            // line.
            render_elements.push(RenderElement::HardLine);
            spacing_context.last_output_end = Some(child.end_byte());
            spacing_context.last_declaration_end = Some(child.end_byte());
            spacing_context.last_declaration_kind = Some(GDScriptNodeKind::SemiColon);
            current_index += 1;
            continue;
        }

        if kind == GDScriptNodeKind::Comment || kind == GDScriptNodeKind::Annotation {
            let previous_byte = pending
                .last()
                .map(|pending_item| pending_item.0.end_byte())
                .or(spacing_context.last_output_end);
            let newlines = previous_byte.map_or(0, |previous_end_byte| {
                count_newlines(source, previous_end_byte, child.start_byte())
            });
            pending.push((child, newlines));
            current_index += 1;
            continue;
        }

        output_pending_before_declaration(
            render_elements,
            input,
            &mut pending,
            &spacing_context,
            child,
        );
        process_node(input, child, render_elements);
        spacing_context.last_output_end = Some(child.end_byte());
        spacing_context.last_declaration_end = Some(child.end_byte());
        spacing_context.last_declaration_kind = Some(kind);
        current_index += 1;
    }

    flush_trailing_pending(
        render_elements,
        input,
        &pending,
        spacing_context.last_output_end,
    );
}

/// Outputs pending comments and annotations collected since the previous
/// declaration, deciding where to put them relative to the upcoming declaration.
///
/// The pending list stores pairs of tree-sitter node and newline count since
/// the previous output. Items that are on consecutive lines without blank gaps
/// are emitted first as we walk toward the next declaration. When we run into an
/// item that is separated by a blank line, we stop and leave the remaining
/// items for the next call.
///
/// There are three ways items can be positioned.
///
/// When trailing comments or annotations in the pending list sit one line
/// before a declaration that needs two blank lines, they are split off and
/// emitted right before that declaration as a leading block.
///
/// When every pending item is an annotation and the next declaration is a
/// variable declaration that does not have its own annotations child, the
/// annotations get merged on the same line as the variable declaration,
/// separated by a space.
///
/// In all other cases the pending block is emitted as its own paragraph
/// between the previous and next declarations, with blank line separation as
/// appropriate for the surrounding declarations.
fn output_pending_before_declaration(
    render_elements: &mut Vec<RenderElement>,
    input: &ParseInput,
    pending: &mut Vec<(tree_sitter::Node, usize)>,
    spacing_context: &TopLevelSpacingContext,
    declaration: tree_sitter::Node,
) {
    let source = input.source;
    let declaration_kind = GDScriptNodeKind::get_kind_from_ast_node(declaration);
    let declaration_start = declaration.start_byte();
    let declaration_needs_two_blank = needs_two_blank_lines(declaration_kind);
    let previous_kind = spacing_context
        .last_declaration_kind
        .unwrap_or(GDScriptNodeKind::Other);
    let previous_needs_two_blank = needs_two_blank_lines(previous_kind);
    let declaration_is_region = declaration_kind == GDScriptNodeKind::RegionStart
        || declaration_kind == GDScriptNodeKind::RegionEnd;
    let previous_is_region = spacing_context.last_declaration_kind
        == Some(GDScriptNodeKind::RegionStart)
        || spacing_context.last_declaration_kind == Some(GDScriptNodeKind::RegionEnd);
    let wants_two_blank_lines = if declaration_is_region || previous_is_region {
        false
    } else {
        previous_needs_two_blank || declaration_needs_two_blank
    };
    let separator_blank_count = calculate_separator_blank_count(
        input,
        previous_kind,
        declaration_kind,
        wants_two_blank_lines,
    );
    let has_previous_content = spacing_context.last_output_end.is_some();

    if pending.is_empty() {
        if !has_previous_content {
            return;
        }
        let newlines = spacing_context
            .last_declaration_end
            .map_or(0, |previous_end_byte| {
                count_newlines(source, previous_end_byte, declaration_start)
            });
        // A semicolon already output its own HardLine; only add blank lines if the
        // source had blank lines after it.
        if spacing_context.last_declaration_kind == Some(GDScriptNodeKind::SemiColon) {
            if newlines >= 2 {
                push_blank_lines(render_elements, separator_blank_count);
            }
            return;
        }
        // Region markers should have no added blank lines.
        if declaration_is_region || previous_is_region {
            push_separator_for_newline_count(newlines, render_elements);
            return;
        }
        // Uses the number of blank lines requested from the configuration when
        // either the previous or current declaration needs them
        // (function/class/constructor). Otherwise preserve the input blank
        // lines up to 1.
        if previous_needs_two_blank || declaration_needs_two_blank {
            push_blank_lines(render_elements, separator_blank_count);
        } else {
            push_separator_for_newline_count(newlines, render_elements);
        }
        return;
    }

    let (last_pending_node, last_pending_newlines) =
        pending.last().expect("pending is non-empty at this point");
    let last_pending_end = last_pending_node.end_byte();
    let newline_count_from_last_pending =
        count_newlines(source, last_pending_end, declaration_start);
    let last_on_new_line = *last_pending_newlines >= 1;
    let mut all_annotations = true;
    let mut pending_index = 0;
    while pending_index < pending.len() {
        if GDScriptNodeKind::get_kind_from_ast_node(pending[pending_index].0)
            != GDScriptNodeKind::Annotation
        {
            all_annotations = false;
            break;
        }
        pending_index += 1;
    }
    let annotation_inline = all_annotations
        && declaration_kind == GDScriptNodeKind::Variable
        && !has_own_annotations_child(declaration)
        && newline_count_from_last_pending == 1;

    // Walk backward from the end of pending: comments on their own line that
    // have no blank lines between them and the next declaration should come
    // right before that declaration.
    let mut leading_count = 0;
    if has_previous_content
        && declaration_needs_two_blank
        && newline_count_from_last_pending == 1
        && last_on_new_line
    {
        let len = pending.len();
        let mut reverse_index = len;
        while reverse_index > 0 {
            reverse_index -= 1;
            let (ref item, newline_count) = pending[reverse_index];
            let item_kind = GDScriptNodeKind::get_kind_from_ast_node(*item);
            if (item_kind != GDScriptNodeKind::Comment && item_kind != GDScriptNodeKind::Annotation)
                || newline_count == 0
            {
                break;
            }
            let zero_blank_to_next = if reverse_index + 1 < len {
                pending[reverse_index + 1].1 == 1
            } else {
                newline_count_from_last_pending == 1
            };
            if !zero_blank_to_next {
                break;
            }
            leading_count += 1;
        }
    }

    let leading_block = if leading_count > 0 {
        let split_at = pending.len() - leading_count;
        pending.split_off(split_at)
    } else {
        Vec::new()
    };

    // Recalculate the number of newlines to the declaration after moving a leading
    // block out of `pending`. When a leading block was split off, `pending` may
    // now be empty (count from the previous declaration) or shorter (count from
    // the new last pending item). When no leading block was split, the original
    // count from the last pending item still applies.
    let newline_count_to_declaration = if pending.is_empty() && !leading_block.is_empty() {
        spacing_context
            .last_declaration_end
            .map_or(0, |last_declaration_end_byte| {
                count_newlines(source, last_declaration_end_byte, declaration_start)
            })
    } else if !leading_block.is_empty() {
        let last_pending_end = pending
            .last()
            .expect("pending is non-empty when leading_block is non-empty")
            .0
            .end_byte();
        count_newlines(source, last_pending_end, declaration_start)
    } else {
        newline_count_from_last_pending
    };

    // This loop only emits items separated by 0 or 1 newlines from the
    // previous one (space or hard line). Unlike `flush_trailing_pending`,
    // which emits every remaining item with a blank line for 2+ newlines,
    // this loop stops as soon as it finds a 2+ newline gap: everything from
    // that point on was already split off into `leading_block` above (a
    // leading block always starts right after a blank-line gap), so the
    // `break` here only ever fires when `leading_count` was 0.
    let mut did_output_anything = false;
    let mut read_position = 0;
    while read_position < pending.len() {
        let newline_count_to_item = pending[read_position].1;
        // The very first item right after previous content in the file needs
        // no separator of its own (there is nothing before it to separate
        // from).
        if has_previous_content || read_position != 0 {
            if newline_count_to_item == 0 {
                render_elements.push(RenderElement::Space);
            } else if newline_count_to_item == 1 {
                render_elements.push(RenderElement::HardLine);
            } else {
                break;
            }
        }
        let (item, _) = pending[read_position];
        process_node(input, item, render_elements);
        did_output_anything = true;
        read_position += 1;
    }
    pending.drain(0..read_position);

    // When the emit loop stopped at a 2+ newline gap, we have remaining
    // pending items that form their own paragraph. Emit them with an
    // appropriate separator before the block.
    let pending_emitted_as_paragraph = !pending.is_empty();
    if pending_emitted_as_paragraph {
        let attached_to_declaration = newline_count_to_declaration == 1;
        let previous_is_class_header = is_class_header(previous_kind);
        if has_previous_content || did_output_anything {
            if attached_to_declaration {
                if wants_two_blank_lines {
                    render_elements.push(RenderElement::BlankLine);
                } else if previous_is_class_header {
                    render_elements.push(RenderElement::HardLine);
                } else {
                    let blank_count = pending[0].1.saturating_sub(1).clamp(1, 2);
                    if blank_count >= 2 {
                        render_elements.push(RenderElement::BlankLine);
                    } else {
                        render_elements.push(RenderElement::HardLine);
                    }
                }
            } else {
                render_elements.push(RenderElement::HardLine);
            }
            render_elements.push(RenderElement::BlankLine);
        }
        emit_pending_block(render_elements, input, pending);
        // pending is reused across declarations, so we need to clear it here
        // now that its contents have been emitted.
        pending.clear();
    }

    // Both pending and leading_block are empty, and the emit loop consumed
    // everything. Just emit the separator before the declaration.
    if !pending_emitted_as_paragraph && leading_block.is_empty() {
        if annotation_inline {
            render_elements.push(RenderElement::Space);
        } else if !has_previous_content {
            if newline_count_to_declaration == 1 {
                render_elements.push(RenderElement::HardLine);
            } else if wants_two_blank_lines {
                push_blank_lines(render_elements, separator_blank_count);
            } else {
                render_elements.push(RenderElement::HardLine);
                render_elements.push(RenderElement::BlankLine);
            }
        } else if wants_two_blank_lines {
            push_blank_lines(render_elements, separator_blank_count);
        } else if newline_count_to_declaration == 1 {
            render_elements.push(RenderElement::HardLine);
        } else {
            render_elements.push(RenderElement::HardLine);
            render_elements.push(RenderElement::BlankLine);
        }
        return;
    }

    // pending was fully consumed but there is a leading_block split off.
    if !pending_emitted_as_paragraph && !leading_block.is_empty() {
        if wants_two_blank_lines {
            push_blank_lines(render_elements, separator_blank_count);
        } else if newline_count_to_declaration <= 1 {
            render_elements.push(RenderElement::HardLine);
        } else {
            render_elements.push(RenderElement::HardLine);
            render_elements.push(RenderElement::BlankLine);
        }
        emit_pending_block(render_elements, input, &leading_block);
        render_elements.push(RenderElement::HardLine);
        return;
    }

    // Here: pending was emitted as a paragraph and we may have a leading
    // block. Emit the trailing separator(s) between the paragraph, the
    // leading block, and the upcoming declaration.
    let attached_to_declaration = newline_count_to_declaration == 1;
    if !leading_block.is_empty() {
        if wants_two_blank_lines {
            push_blank_lines(render_elements, separator_blank_count);
        } else {
            render_elements.push(RenderElement::HardLine);
        }
        render_elements.push(RenderElement::BlankLine);
        emit_pending_block(render_elements, input, &leading_block);
        render_elements.push(RenderElement::HardLine);
    } else if attached_to_declaration {
        render_elements.push(RenderElement::HardLine);
    } else if declaration_needs_two_blank {
        let declaration_blank_count =
            get_blank_line_count_before_declaration(input, declaration_kind);
        push_blank_lines(render_elements, declaration_blank_count);
    } else {
        render_elements.push(RenderElement::HardLine);
        render_elements.push(RenderElement::BlankLine);
    }
}

/// Outputs a batch of buffered comments or annotations. Iterates through the
/// pending list and inserts spaces, hard lines, or blank lines between items
/// based on the newline counts recorded when they were buffered.
fn emit_pending_block(
    render_elements: &mut Vec<RenderElement>,
    input: &ParseInput,
    pending: &[(tree_sitter::Node, usize)],
) {
    let len = pending.len();
    let mut pending_index = 0;
    while pending_index < len {
        let (child, newlines) = pending[pending_index];
        if pending_index > 0 {
            push_separator_for_newline_count(newlines, render_elements);
        }
        process_node(input, child, render_elements);
        pending_index += 1;
    }
}

/// Outputs any remaining comments or annotations that trailed after the last
/// statement, with spacing based on the source newline count between them. Runs
/// at the end of the file, after all statements are processed.
fn flush_trailing_pending(
    render_elements: &mut Vec<RenderElement>,
    input: &ParseInput,
    pending_ast_nodes: &[(tree_sitter::Node, usize)],
    last_output_end: Option<usize>,
) {
    let mut has_output_any = false;
    let mut current_node_index = 0;
    while current_node_index < pending_ast_nodes.len() {
        let newline_count_to_item = pending_ast_nodes[current_node_index].1;
        // The very first item right after previous content in the file needs
        // no separator of its own (there is nothing before it to separate
        // from).
        if has_output_any || last_output_end.is_some() {
            push_separator_for_newline_count(newline_count_to_item, render_elements);
        }
        let (item, _) = pending_ast_nodes[current_node_index];
        process_node(input, item, render_elements);
        has_output_any = true;
        current_node_index += 1;
    }
}

/// Decides and inserts spacing (blank lines / space / newline) between two
/// sibling nodes in a body block (class body, function body, etc.).
///
/// The function chooses spacing based on:
///
/// - the kind of previous and current child (comment vs non-comment vs declaration)
/// - the number of newlines between AST nodes in the original source code
/// - whether the current node needs structured blank lines (needs_two_blank parameter)
///
/// Special cases:
///
/// - comment followed by a comment: preserve blank lines, max at 1.
/// - comment followed by a non-comment node: attach the comment to the next
///   line if the source code had no blank line, otherwise use the target count if
///   needs_two_blank is true.
/// - non-comment followed by non-comment: use target count if needs_two_blank,
///   otherwise use 1 newline.
/// - other cases: preserve up to 1 blank line (e.g. if source had 2+ blank
///   lines), else insert one newline.
fn add_spacing_between_body_children(
    previous_end: usize,
    current_start: usize,
    input: &ParseInput,
    render_elements: &mut Vec<RenderElement>,
    previous_kind: Option<GDScriptNodeKind>,
    current_kind: GDScriptNodeKind,
    needs_two_blank: bool,
) {
    let source = input.source;
    let newline_count = count_newlines(source, previous_end, current_start);
    let current_blank_count = get_blank_line_count_before_declaration(input, current_kind);
    let target_blank_line_count = if current_blank_count == 0 {
        input.blank_lines_around_definitions
    } else {
        current_blank_count
    };
    if newline_count == 0 {
        render_elements.push(RenderElement::Space);
    } else if previous_kind == Some(GDScriptNodeKind::Comment)
        && current_kind == GDScriptNodeKind::Comment
    {
        // Series of comments: preserve up to 1 blank line.
        render_elements.push(RenderElement::HardLine);
        if newline_count != 1 {
            render_elements.push(RenderElement::BlankLine);
        }
    } else if previous_kind == Some(GDScriptNodeKind::Comment) {
        // Comment followed by non-comment node: attach comment to next line if
        // there are 0 blank lines between them in source, else use target
        // count.
        if newline_count == 1 {
            render_elements.push(RenderElement::HardLine);
        } else if needs_two_blank {
            push_blank_lines(render_elements, target_blank_line_count);
        } else {
            // this is a comment before something other than a declaration, fall
            // back to limiting to 1 empty line.
            render_elements.push(RenderElement::HardLine);
            render_elements.push(RenderElement::BlankLine);
        }
    } else if needs_two_blank {
        push_blank_lines(render_elements, target_blank_line_count);
    } else if newline_count >= 2 {
        // the node is neither a comment nor a declaration, with 2+ newlines in
        // source. We limit to 1 blank line.
        render_elements.push(RenderElement::HardLine);
        render_elements.push(RenderElement::BlankLine);
    } else {
        render_elements.push(RenderElement::HardLine);
    }
}

/// Formats a setget node (setter/getter definitions). Outputs the first child
/// (the value), then indents and formats the setter/getter bodies on separate
/// lines.
fn process_setget(
    input: &ParseInput,
    node: tree_sitter::Node,
    render_elements: &mut Vec<RenderElement>,
) {
    let child_count = node.child_count();
    if child_count == 0 {
        return;
    }
    if let Some(first) = node.child(0) {
        process_node(input, first, render_elements);
    }
    if child_count > 1 {
        render_elements.push(RenderElement::HardLine);
        let indent_index = begin_indent(render_elements, 1);
        let mut inner = 1usize;
        let mut previous: Option<tree_sitter::Node> = None;
        while inner < child_count {
            if let Some(inner_child) = node.child(inner as u32) {
                if let Some(ref previous_child) = previous {
                    emit_inter_child_separator(
                        GDScriptNodeKind::SetGet,
                        previous_child,
                        &inner_child,
                        render_elements,
                    );
                }
                process_node(input, inner_child, render_elements);
                previous = Some(inner_child);
            }
            inner += 1;
        }
        finish_indent(render_elements, indent_index);
    }
}

/// Formats comma-separated containers (arrays, dictionaries, enum lists,
/// function parameters, function call arguments). Wraps content in a Group so
/// the renderer can choose between single-line or multi-line output. Optionally
/// adds trailing commas, handles placing inline comments after commas, and
/// handles empty or single-element containers too which render inline
fn process_container(
    input: &ParseInput,
    node: tree_sitter::Node,
    render_elements: &mut Vec<RenderElement>,
) {
    let node_kind = GDScriptNodeKind::get_kind_from_ast_node(node);
    let child_count = node.child_count();
    if child_count < 3 {
        if let Some(open) = node.child(0) {
            process_node(input, open, render_elements);
        }
        if node_kind == GDScriptNodeKind::Dictionary
            || node_kind == GDScriptNodeKind::EnumeratorList
        {
            render_elements.push(RenderElement::Space);
        }
        if let Some(close) = node.child(1) {
            process_node(input, close, render_elements);
        }
        return;
    }

    let group_index = begin_group(render_elements);

    if let Some(open) = node.child(0) {
        process_node(input, open, render_elements);
    }
    let needs_inner_space =
        node_kind == GDScriptNodeKind::EnumeratorList || node_kind == GDScriptNodeKind::Dictionary;
    if needs_inner_space {
        render_elements.push(RenderElement::SpaceSingleLineOnly);
    }
    render_elements.push(RenderElement::SoftLine);

    let uses_double_hanging_indent =
        node_kind == GDScriptNodeKind::Parameters || node_kind == GDScriptNodeKind::Arguments;
    let indent_level: u16 = if uses_double_hanging_indent {
        input.continuation_indent_level
    } else {
        1
    };
    let indent_index = begin_indent(render_elements, indent_level);

    let mut has_comment = false;
    let mut last_was_comment = false;
    let mut trailing_comma_handled = false;
    let mut index = 1usize;
    let mut previous: Option<tree_sitter::Node> = None;
    let mut skip_next_separator = false;
    while index < child_count - 1 {
        if let Some(child) = node.child(index as u32) {
            let child_kind = GDScriptNodeKind::get_kind_from_ast_node(child);
            if index == child_count - 2 && child_kind == GDScriptNodeKind::TokenComma {
                index += 1;
                continue;
            }
            let is_comment = child_kind == GDScriptNodeKind::Comment;
            if is_comment && index == child_count - 2 {
                let previous_kind = previous.map_or(GDScriptNodeKind::Other, |previous_child| {
                    GDScriptNodeKind::get_kind_from_ast_node(previous_child)
                });
                // Skip inserting a trailing comma before a trailing comment when
                // the previous element is a comma (already trailing) or another
                // comment (comment block, not an element needing a comma).
                if previous_kind != GDScriptNodeKind::TokenComma
                    && previous_kind != GDScriptNodeKind::Comment
                {
                    let text_index = render_elements.len() + 1;
                    render_elements.push(RenderElement::Branch {
                        if_single_line: None,
                        if_multiline: Some(RangeRenderElement {
                            start: text_index,
                            end: text_index + 1,
                        }),
                    });
                    render_elements.push(RenderElement::TextStatic(","));
                    render_elements.push(RenderElement::Space);
                    skip_next_separator = true;
                    trailing_comma_handled = true;
                }
            }
            if is_comment {
                has_comment = true;
            }
            if !skip_next_separator {
                if let Some(ref previous_child) = previous {
                    let previous_kind = GDScriptNodeKind::get_kind_from_ast_node(*previous_child);
                    if previous_kind == GDScriptNodeKind::Comment {
                        render_elements.push(RenderElement::HardLine);
                    } else {
                        emit_inter_child_separator(
                            node_kind,
                            previous_child,
                            &child,
                            render_elements,
                        );
                    }
                }
            }
            skip_next_separator = false;
            process_node(input, child, render_elements);
            if child_kind == GDScriptNodeKind::TokenComma {
                let mut next_is_comment = false;
                let mut next_same_line = false;
                if index + 1 < child_count {
                    if let Some(next) = node.child((index + 1) as u32) {
                        next_is_comment = GDScriptNodeKind::get_kind_from_ast_node(next)
                            == GDScriptNodeKind::Comment;
                        if next_is_comment {
                            next_same_line =
                                !has_newline(input.source, child.end_byte(), next.start_byte());
                        }
                    }
                }
                if next_is_comment {
                    if next_same_line {
                        render_elements.push(RenderElement::Space);
                    } else {
                        render_elements.push(RenderElement::HardLine);
                    }
                } else {
                    render_elements.push(RenderElement::SoftLine);
                    render_elements.push(RenderElement::SpaceSingleLineOnly);
                }
                skip_next_separator = true;
            }
            previous = Some(child);
            last_was_comment = is_comment;
        }
        index += 1;
    }

    // A container's children are: [open_delimiter, elements_and_commas...,
    // close_delimiter]. The minimal single-element form is 3 children
    // ([open, element, close]); child_count >= 4 means there is either a
    // trailing comma on a single element or more than one element. In both
    // cases we ensure a trailing comma in multiline layout and check the source
    // for a forced line break.
    const MIN_CHILD_COUNT_BEYOND_SINGLE_ELEMENT: usize = 4;
    let contains_more_than_one_element = child_count >= MIN_CHILD_COUNT_BEYOND_SINGLE_ELEMENT;
    if contains_more_than_one_element && !trailing_comma_handled && !last_was_comment {
        let text_index = render_elements.len() + 1;
        render_elements.push(RenderElement::Branch {
            if_single_line: None,
            if_multiline: Some(RangeRenderElement {
                start: text_index,
                end: text_index + 1,
            }),
        });
        render_elements.push(RenderElement::TextStatic(","));
    }

    if contains_more_than_one_element && let Some(open) = node.child(0) {
        let close_byte = node
            .child((child_count - 1) as u32)
            .expect("container node has close delimiter")
            .start_byte();
        if has_newline(input.source, open.end_byte(), close_byte) {
            render_elements.push(RenderElement::ForceBreakingParent);
        }
    }

    finish_indent(render_elements, indent_index);

    if has_comment {
        render_elements.push(RenderElement::HardLine);
    } else {
        render_elements.push(RenderElement::SoftLine);
    }
    if needs_inner_space {
        render_elements.push(RenderElement::SpaceSingleLineOnly);
    }

    if let Some(close) = node.child((child_count - 1) as u32) {
        process_node(input, close, render_elements);
    }

    finish_group(render_elements, group_index);
}

/// Formats ParenthesizedExpression nodes with a Group. Falls back to
/// process_children_with_spacing for single-line expressions or when the inner
/// content already handles its own indentation (lambdas, arrays, dicts).
fn process_parenthesized_expression(
    input: &ParseInput,
    node: tree_sitter::Node,
    render_elements: &mut Vec<RenderElement>,
) {
    let child_count = node.child_count();
    if child_count < 3 {
        process_children_with_spacing(input, node, render_elements);
        return;
    }

    let inner_self_indents = if let Some(first_inner) = node.child(1) {
        matches!(
            GDScriptNodeKind::get_kind_from_ast_node(first_inner),
            GDScriptNodeKind::Lambda | GDScriptNodeKind::Array | GDScriptNodeKind::Dictionary
        )
    } else {
        false
    };
    if inner_self_indents {
        process_children_with_spacing(input, node, render_elements);
        return;
    }

    let body_has_newlines = {
        if let (Some(open_node), Some(close_node)) =
            (node.child(0), node.child((child_count - 1) as u32))
        {
            has_newline(input.source, open_node.end_byte(), close_node.start_byte())
        } else {
            false
        }
    };
    if !body_has_newlines {
        process_children_with_spacing(input, node, render_elements);
        return;
    }

    let group_index = begin_group(render_elements);

    if let Some(open) = node.child(0) {
        process_node(input, open, render_elements);
    }

    render_elements.push(RenderElement::SoftLine);
    let indent_index = begin_indent(render_elements, 1);

    let end = (child_count - 1) as u32;
    let mut index: u32 = 1;
    let mut previous: Option<tree_sitter::Node> = None;
    while index < end {
        if let Some(child) = node.child(index) {
            if let Some(ref previous_child) = previous {
                emit_inter_child_separator(
                    GDScriptNodeKind::ParenthesizedExpression,
                    previous_child,
                    &child,
                    render_elements,
                );
            }
            process_node(input, child, render_elements);
            previous = Some(child);
        }
        index += 1;
    }

    finish_indent(render_elements, indent_index);
    render_elements.push(RenderElement::SoftLine);

    if let Some(close) = node.child((child_count - 1) as u32) {
        process_node(input, close, render_elements);
    }

    finish_group(render_elements, group_index);
}

/// Formats BinaryOperator nodes. For and/or chains, flattens left-associative
/// operators into balanced chunks to avoid deep nesting. For other operators,
/// wraps the expression in a Group that breaks on newlines.
fn process_binary_operator(
    input: &ParseInput,
    node: tree_sitter::Node,
    render_elements: &mut Vec<RenderElement>,
) {
    let child_count = node.child_count();
    if child_count < 3 {
        process_children_with_spacing(input, node, render_elements);
        return;
    }

    let is_and_or = if let Some(operator_node) = node.child(1) {
        let text = &input.source[operator_node.start_byte()..operator_node.end_byte()];
        text == "and" || text == "or"
    } else {
        false
    };

    if is_and_or {
        // Flatten left-associative and/or chain into segments.
        // Each segment = (optional_operator_node, expression_node).
        let mut segments: Vec<(Option<tree_sitter::Node>, tree_sitter::Node)> =
            Vec::with_capacity(child_count);
        let mut current = node;
        let mut levels: Vec<tree_sitter::Node> = Vec::with_capacity(child_count);

        while let Some(left) = current.child(0) {
            if GDScriptNodeKind::get_kind_from_ast_node(left) == GDScriptNodeKind::BinaryOperator {
                let left_operator_text = if let Some(operator_node) = left.child(1) {
                    &input.source[operator_node.start_byte()..operator_node.end_byte()]
                } else {
                    ""
                };
                let current_operator_text = if let Some(operator_node) = current.child(1) {
                    &input.source[operator_node.start_byte()..operator_node.end_byte()]
                } else {
                    ""
                };
                if left_operator_text == current_operator_text {
                    levels.push(current);
                    current = left;
                    continue;
                }
            }
            break;
        }

        if let Some(left_expr) = current.child(0) {
            segments.push((None, left_expr));
        }
        if let Some(right_expr) = current.child(2) {
            segments.push((current.child(1), right_expr));
        }
        let mut level_index = levels.len();
        while level_index > 0 {
            level_index -= 1;
            if let Some(right_expr) = levels[level_index].child(2) {
                segments.push((levels[level_index].child(1), right_expr));
            }
        }

        if segments.len() >= 2 {
            // Estimate total rendered width from source spans.
            let mut total = 0usize;
            let mut segment_index = 0;
            while segment_index < segments.len() {
                let (operator_node, expr) = &segments[segment_index];
                if segment_index > 0 {
                    if let Some(op) = operator_node {
                        total += node_byte_length(*op);
                    }
                    total += 1;
                }
                total += node_byte_length(*expr);
                segment_index += 1;
            }

            if total > input.max_line_length {
                // Try 2 segments per line, else fall back to 1.
                let chunk_size = {
                    let available = input.max_line_length;
                    let segment_count = segments.len();
                    let mut fits = true;
                    let mut chunk_start = 0;
                    while chunk_start < segment_count {
                        let chunk_end = (chunk_start + 2).min(segment_count);
                        let mut chunk_width = 0usize;
                        let mut segment_index = chunk_start;
                        while segment_index < chunk_end {
                            let (operator, expression) = &segments[segment_index];
                            if segment_index > 0 {
                                if let Some(op) = operator {
                                    chunk_width += node_byte_length(*op);
                                }
                                chunk_width += 1;
                            }
                            chunk_width += node_byte_length(*expression);
                            if segment_index < chunk_end - 1 {
                                chunk_width += 1;
                            }
                            segment_index += 1;
                        }
                        if chunk_width > available {
                            fits = false;
                            break;
                        }
                        chunk_start = chunk_end;
                    }
                    if fits { 2 } else { 1 }
                };

                // Build doc with balanced chunks. Only wrap in parens when
                // not already inside a parenthesized_expression.
                let parent_is_paren = if let Some(parent_node) = node.parent() {
                    GDScriptNodeKind::get_kind_from_ast_node(parent_node)
                        == GDScriptNodeKind::ParenthesizedExpression
                } else {
                    false
                };
                if !parent_is_paren {
                    render_elements.push(RenderElement::TextStatic("("));
                    render_elements.push(RenderElement::HardLine);
                }
                let indent_index = if !parent_is_paren {
                    Some(begin_indent(render_elements, 1))
                } else {
                    None
                };
                let segment_count = segments.len();
                let mut chunk_start = 0;
                while chunk_start < segment_count {
                    let chunk_end = (chunk_start + chunk_size).min(segment_count);
                    let mut segment_index = chunk_start;
                    while segment_index < chunk_end {
                        let (operator_node, expr) = &segments[segment_index];
                        if segment_index > 0 {
                            if let Some(op) = operator_node {
                                render_elements.push(RenderElement::Text {
                                    range: RangeSourceBytes {
                                        start_byte: op.start_byte(),
                                        end_byte: op.end_byte(),
                                    },
                                });
                            }
                            render_elements.push(RenderElement::Space);
                        }
                        process_node(input, *expr, render_elements);
                        if segment_index < chunk_end - 1 {
                            render_elements.push(RenderElement::Space);
                        }
                        segment_index += 1;
                    }
                    chunk_start = chunk_end;
                    if chunk_start < segment_count {
                        render_elements.push(RenderElement::HardLine);
                    }
                }
                if let Some(indent_position) = indent_index {
                    finish_indent(render_elements, indent_position);
                }
                if !parent_is_paren {
                    render_elements.push(RenderElement::HardLine);
                    render_elements.push(RenderElement::TextStatic(")"));
                }
                return;
            }
        }
    }

    let mut has_line_continuation = false;
    let mut current_index = 0;
    while current_index < child_count as u32 {
        if let Some(c) = node.child(current_index) {
            if GDScriptNodeKind::get_kind_from_ast_node(c) == GDScriptNodeKind::LineContinuation {
                has_line_continuation = true;
                break;
            }
        }
        current_index += 1;
    }
    if has_line_continuation || !has_newline(input.source, node.start_byte(), node.end_byte()) {
        process_children_with_spacing(input, node, render_elements);
        return;
    }

    let group_index = begin_group(render_elements);

    if let Some(left) = node.child(0) {
        process_node(input, left, render_elements);
    }

    render_elements.push(RenderElement::SoftLine);
    let space_index = render_elements.len() + 1;
    render_elements.push(RenderElement::Branch {
        if_single_line: Some(RangeRenderElement {
            start: space_index,
            end: space_index + 1,
        }),
        if_multiline: None,
    });
    render_elements.push(RenderElement::Space);

    // Process the right side of this binary expression
    if let Some(op) = node.child(1) {
        process_node(input, op, render_elements);
    }

    if let Some(right) = node.child(2) {
        render_elements.push(RenderElement::Space);
        process_node(input, right, render_elements);
    }

    finish_group(render_elements, group_index);
}

/// Formats Condition nodes (ternary if/else expressions) with a Group. Each
/// part (value, condition, alternate) goes on its own line when the expression
/// spans multiple lines.
fn process_conditional_expression(
    input: &ParseInput,
    node: tree_sitter::Node,
    render_elements: &mut Vec<RenderElement>,
) {
    let child_count = node.child_count();
    if child_count < 5 {
        process_children_with_spacing(input, node, render_elements);
        return;
    }
    if !has_newline(input.source, node.start_byte(), node.end_byte()) {
        process_children_with_spacing(input, node, render_elements);
        return;
    }

    let group_index = begin_group(render_elements);
    render_elements.push(RenderElement::ForceBreakingParent);

    if let Some(value) = node.child(0) {
        process_node(input, value, render_elements);
    }

    render_elements.push(RenderElement::SoftLine);
    let if_space_index = render_elements.len() + 1;
    render_elements.push(RenderElement::Branch {
        if_single_line: Some(RangeRenderElement {
            start: if_space_index,
            end: if_space_index + 1,
        }),
        if_multiline: None,
    });
    render_elements.push(RenderElement::Space);

    if let Some(if_kw) = node.child(1) {
        process_node(input, if_kw, render_elements);
    }

    if let Some(cond) = node.child(2) {
        render_elements.push(RenderElement::Space);
        process_node(input, cond, render_elements);
    }

    render_elements.push(RenderElement::SoftLine);
    let else_space_index = render_elements.len() + 1;
    render_elements.push(RenderElement::Branch {
        if_single_line: Some(RangeRenderElement {
            start: else_space_index,
            end: else_space_index + 1,
        }),
        if_multiline: None,
    });
    render_elements.push(RenderElement::Space);

    if let Some(else_kw) = node.child(3) {
        process_node(input, else_kw, render_elements);
    }

    if let Some(value) = node.child(4) {
        render_elements.push(RenderElement::Space);
        process_node(input, value, render_elements);
    }

    finish_group(render_elements, group_index);
}

/// Formats Attribute nodes (dot-access chains like a.b.c()). Handles line
/// continuations and wraps long chains onto separate lines. Uses
/// process_method_call_flat for attribute call nodes in the chain.
fn process_attribute(
    input: &ParseInput,
    node: tree_sitter::Node,
    render_elements: &mut Vec<RenderElement>,
) {
    let child_count = node.child_count();
    // Only handle dot-access chains (child_count >= 5: at least 2 method calls).
    // Single method calls like a.foo() go through process_children_with_spacing.
    let is_dot_chain = if let Some(c) = node.child(1) {
        GDScriptNodeKind::get_kind_from_ast_node(c) == GDScriptNodeKind::TokenDot
    } else {
        false
    };
    if child_count < 5 || !is_dot_chain {
        process_children_with_spacing(input, node, render_elements);
        return;
    }

    // GDScript does not support function call chains as a standalone statement
    // without explicit line continuations, but actually it supports chains in
    // some contexts like parenthesized expressions. We check for those two
    // cases here and adjust the formatting accordingly to make sure automatic
    // line wrapping works.
    //
    // This is valid gdscript:
    // ("test"
    //       .begins_with("t"))
    //
    // But this, without parentheses or brackets, gives an error:
    //
    // "test"
    //       .begins_with("t")
    //
    // You need a continuation line break here:
    //
    // "test" \
    //       .begins_with("t")
    let mut allows_implicit_continuation = false;
    let mut visited_ancestor = node.parent();
    while let Some(current_ancestor) = visited_ancestor {
        let current_ancestor_kind = GDScriptNodeKind::get_kind_from_ast_node(current_ancestor);
        if matches!(
            current_ancestor_kind,
            GDScriptNodeKind::Array
                | GDScriptNodeKind::Dictionary
                | GDScriptNodeKind::Arguments
                | GDScriptNodeKind::SubscriptArguments
                | GDScriptNodeKind::ParenthesizedExpression
        ) {
            allows_implicit_continuation = true;
            break;
        }
        if matches!(
            current_ancestor_kind,
            GDScriptNodeKind::Assignment
                | GDScriptNodeKind::AugmentedAssignment
                | GDScriptNodeKind::ExpressionStatement
                | GDScriptNodeKind::ReturnStatement
                | GDScriptNodeKind::Body
        ) {
            break;
        }
        visited_ancestor = current_ancestor.parent();
    }

    let mut has_explicit_line_continuation = false;
    let mut child_index = 1;
    while child_index < child_count {
        if let Some(child) = node.child(child_index as u32)
            && GDScriptNodeKind::get_kind_from_ast_node(child) == GDScriptNodeKind::LineContinuation
        {
            has_explicit_line_continuation = true;
            break;
        }
        child_index += 1;
    }

    let group_index = begin_group(render_elements);

    if let Some(expr) = node.child(0) {
        process_node(input, expr, render_elements);
    }

    let chain_indent_level = if allows_implicit_continuation || has_explicit_line_continuation {
        0
    } else {
        input.continuation_indent_level
    };
    let continuation_indent_index = begin_indent(render_elements, chain_indent_level);
    let mut attribute_index: u32 = 1;
    while attribute_index < child_count as u32 {
        let child = node.child(attribute_index);
        let next = node.child(attribute_index + 1);

        let is_line_continuation = if let Some(child_node) = child {
            GDScriptNodeKind::get_kind_from_ast_node(child_node)
                == GDScriptNodeKind::LineContinuation
        } else {
            false
        };
        if is_line_continuation {
            render_elements.push(RenderElement::Space);
            process_node(
                input,
                child.expect("line_continuation child exists"),
                render_elements,
            );
            attribute_index += 1;
            let dot_after_lc = node.child(attribute_index);
            let call_after_lc = node.child(attribute_index + 1);
            if let Some(dot_node) = dot_after_lc {
                let continuation_indent_index =
                    begin_indent(render_elements, input.continuation_indent_level);
                process_node(input, dot_node, render_elements);
                if let Some(call_node) = call_after_lc {
                    if GDScriptNodeKind::get_kind_from_ast_node(call_node)
                        == GDScriptNodeKind::AttributeCall
                    {
                        process_method_call_flat(input, call_node, render_elements);
                    } else {
                        process_node(input, call_node, render_elements);
                    }
                }
                finish_indent(render_elements, continuation_indent_index);
            }
            attribute_index += 2;
            continue;
        }

        if !allows_implicit_continuation && !has_explicit_line_continuation {
            let continuation_index = render_elements.len() + 1;
            render_elements.push(RenderElement::Branch {
                if_single_line: None,
                if_multiline: Some(RangeRenderElement {
                    start: continuation_index,
                    end: continuation_index + 2,
                }),
            });
            render_elements.push(RenderElement::Space);
            render_elements.push(RenderElement::TextStatic("\\"));
        }
        if !has_explicit_line_continuation {
            render_elements.push(RenderElement::SoftLine);
        }

        if let Some(dot_node) = child {
            process_node(input, dot_node, render_elements);
        }
        if let Some(call_node) = next {
            if GDScriptNodeKind::get_kind_from_ast_node(call_node)
                == GDScriptNodeKind::AttributeCall
            {
                process_method_call_flat(input, call_node, render_elements);
            } else {
                process_node(input, call_node, render_elements);
            }
        }

        attribute_index += 2;
    }
    finish_indent(render_elements, continuation_indent_index);
    finish_group(render_elements, group_index);
}

/// Builds a method call (name + arguments) inline without extra indentation.
/// Used inside a dot-access chain. Uses emit_inter_child_separator between
/// arguments and between the last argument and the closing parenthesis.
fn process_method_call_flat(
    input: &ParseInput,
    attribute_call: tree_sitter::Node,
    render_elements: &mut Vec<RenderElement>,
) {
    if let Some(method_name) = attribute_call.child(0) {
        process_node(input, method_name, render_elements);
    }
    if let Some(args) = attribute_call.child(1) {
        let argument_child_count = args.child_count();
        if argument_child_count >= 2 {
            if let Some(open) = args.child(0) {
                process_node(input, open, render_elements);
            }
            let close_parenthesis_index = (argument_child_count - 1) as u32;
            let args_kind = GDScriptNodeKind::get_kind_from_ast_node(args);
            let has_trailing_comma = if close_parenthesis_index >= 2 {
                if let Some(node_before_close_paren) = args.child(close_parenthesis_index - 1) {
                    GDScriptNodeKind::get_kind_from_ast_node(node_before_close_paren)
                        == GDScriptNodeKind::TokenComma
                } else {
                    false
                }
            } else {
                false
            };
            let body_end = if has_trailing_comma {
                close_parenthesis_index - 1
            } else {
                close_parenthesis_index
            };
            let mut previous_child: Option<tree_sitter::Node> =
                Some(args.child(0).expect("argument_child_count>=2"));
            let mut argument_index: u32 = 1;
            while argument_index < body_end {
                if let Some(child_argument) = args.child(argument_index) {
                    if let Some(ref previous_node) = previous_child {
                        emit_inter_child_separator(
                            args_kind,
                            previous_node,
                            &child_argument,
                            render_elements,
                        );
                    }
                    process_node(input, child_argument, render_elements);
                    previous_child = Some(child_argument);
                }
                argument_index += 1;
            }
            if let Some(close) = args.child(close_parenthesis_index) {
                if let Some(ref previous_node) = previous_child {
                    emit_inter_child_separator(args_kind, previous_node, &close, render_elements);
                }
                process_node(input, close, render_elements);
            }
        }
    }
}

/// Formats Lambda nodes with a Group for flat/break layout. Uses
/// emit_lambda_separator between lambda children. Applies ForceBreak when the
/// lambda body spans multiple lines.
fn process_lambda(
    input: &ParseInput,
    node: tree_sitter::Node,
    render_elements: &mut Vec<RenderElement>,
) {
    let child_count = node.child_count();
    if child_count == 0 {
        return;
    }

    let group_index = begin_group(render_elements);

    let mut index = 0;
    let mut previous: Option<tree_sitter::Node> = None;
    while index < child_count {
        if let Some(child) = node.child(index as u32) {
            let child_kind = GDScriptNodeKind::get_kind_from_ast_node(child);
            if let Some(ref previous_child) = previous {
                let previous_kind = GDScriptNodeKind::get_kind_from_ast_node(*previous_child);
                if child_kind == GDScriptNodeKind::Body
                    && previous_kind == GDScriptNodeKind::TokenColon
                {
                    render_elements.push(RenderElement::SoftLine);
                    let space_index = render_elements.len() + 1;
                    render_elements.push(RenderElement::Branch {
                        if_single_line: Some(RangeRenderElement {
                            start: space_index,
                            end: space_index + 1,
                        }),
                        if_multiline: None,
                    });
                    render_elements.push(RenderElement::Space);
                } else {
                    emit_lambda_separator(previous_kind, child_kind, render_elements);
                }
            }
            process_node(input, child, render_elements);
            previous = Some(child);
        }
        index += 1;
    }

    // A lambda's body spans multiple lines exactly when its source range
    // contains a newline, so a single scan can drive both decisions below:
    // forcing the enclosing group to break, and (when the lambda sits inside
    // parentheses) adding a trailing hard line after the body.
    let mut body_is_multiline = false;
    let mut current_index: u32 = 0;
    while current_index < child_count as u32 {
        if let Some(child) = node.child(current_index) {
            if GDScriptNodeKind::get_kind_from_ast_node(child) == GDScriptNodeKind::Body
                && has_newline(input.source, child.start_byte(), child.end_byte())
            {
                body_is_multiline = true;
                render_elements.push(RenderElement::ForceBreakingParent);
                break;
            }
        }
        current_index += 1;
    }

    let parent_is_paren = if let Some(parent_node) = node.parent() {
        GDScriptNodeKind::get_kind_from_ast_node(parent_node)
            == GDScriptNodeKind::ParenthesizedExpression
    } else {
        false
    };
    if body_is_multiline && parent_is_paren {
        render_elements.push(RenderElement::HardLine);
    }

    finish_group(render_elements, group_index);
}

/// Handles spacing between children of a lambda node. Deals with comments (hard
/// line), body-like nodes (hard line), specific token pairs that need no
/// separator, and defaults to a space.
fn emit_lambda_separator(
    previous_kind: GDScriptNodeKind,
    current_kind: GDScriptNodeKind,
    render_elements: &mut Vec<RenderElement>,
) {
    if previous_kind == GDScriptNodeKind::Comment {
        render_elements.push(RenderElement::HardLine);
        return;
    }

    if current_kind == GDScriptNodeKind::TokenParen
        || current_kind == GDScriptNodeKind::TokenBracket
        || current_kind == GDScriptNodeKind::TokenBrace
        || current_kind == GDScriptNodeKind::TokenColon
    {
        return;
    }

    if current_kind == GDScriptNodeKind::Body
        || current_kind == GDScriptNodeKind::ClassBody
        || current_kind == GDScriptNodeKind::MatchBody
    {
        render_elements.push(RenderElement::HardLine);
        return;
    }

    if previous_kind == GDScriptNodeKind::KeywordFunc
        && current_kind == GDScriptNodeKind::Parameters
    {
        return;
    }

    render_elements.push(RenderElement::Space);
}

/// Fallback formatter for any node kind without a dedicated builder. Also used
/// by other builders as a passthrough when they decide not to apply special
/// formatting. Iterates over all children and uses emit_inter_child_separator
/// to decide spacing between them. Handles line continuation tokens specially.
fn process_children_with_spacing(
    input: &ParseInput,
    node: tree_sitter::Node,
    render_elements: &mut Vec<RenderElement>,
) {
    let parent_kind = GDScriptNodeKind::get_kind_from_ast_node(node);
    let child_count = node.child_count();
    let mut index = 0;
    let mut previous: Option<tree_sitter::Node> = None;
    while index < child_count {
        if let Some(child) = node.child(index as u32) {
            // This code is similar to the one in process_body(). See comments
            // there for some explanation of what this does and why it's needed.
            match classify_disabled_region_overlap(input, node, child, index) {
                DisabledRegionOverlapKind::CoveredFully(disabled_run) => {
                    let region = disabled_run.region;
                    if child.start_byte() == region.start {
                        if let Some(ref previous_child) = previous {
                            emit_inter_child_separator(
                                parent_kind,
                                previous_child,
                                &child,
                                render_elements,
                            );
                        }
                        render_elements.push(RenderElement::UnformattedSource {
                            range: RangeSourceBytes {
                                start_byte: region.start,
                                end_byte: region.end,
                            },
                        });
                    }
                    let last_covered_child = node
                        .child(disabled_run.last_covered_index as u32)
                        .expect("last_covered_index came from this same node's children");
                    previous = Some(last_covered_child);
                    index = disabled_run.last_covered_index + 1;
                    continue;
                }
                DisabledRegionOverlapKind::PartiallyCovered => {
                    process_node(input, child, render_elements);
                    previous = Some(child);
                    index += 1;
                    continue;
                }
                DisabledRegionOverlapKind::None => {}
            }

            let child_kind = GDScriptNodeKind::get_kind_from_ast_node(child);
            if let Some(ref previous_child) = previous {
                let previous_kind = GDScriptNodeKind::get_kind_from_ast_node(*previous_child);
                emit_inter_child_separator(parent_kind, previous_child, &child, render_elements);
                if previous_kind == GDScriptNodeKind::LineContinuation {
                    let indent_index =
                        begin_indent(render_elements, input.continuation_indent_level);
                    process_node(input, child, render_elements);
                    finish_indent(render_elements, indent_index);
                    previous = Some(child);
                    index += 1;
                    continue;
                }
            }
            if child_kind == GDScriptNodeKind::LineContinuation {
                let start = child.start_byte();
                render_elements.push(RenderElement::Text {
                    range: RangeSourceBytes {
                        start_byte: start,
                        end_byte: start + 1,
                    },
                });
                render_elements.push(RenderElement::HardLine);
                previous = Some(child);
                index += 1;
                continue;
            }
            process_node(input, child, render_elements);
            previous = Some(child);
        }
        index += 1;
    }
}

/// Decides what goes between any two sibling AST nodes: a space, a newline,
/// a blank line, or nothing. Handles tokens (parens, brackets, dots, commas,
/// colons), body nodes, comments, annotations, and special parent cases like
/// InferredType and UnaryOperator.
fn emit_inter_child_separator(
    parent_kind: GDScriptNodeKind,
    previous_child: &tree_sitter::Node,
    current: &tree_sitter::Node,
    render_elements: &mut Vec<RenderElement>,
) {
    let previous_kind = GDScriptNodeKind::get_kind_from_ast_node(*previous_child);
    let current_kind = GDScriptNodeKind::get_kind_from_ast_node(*current);

    if parent_kind == GDScriptNodeKind::InferredType {
        return;
    }

    if current_kind == GDScriptNodeKind::ElifStatement
        || current_kind == GDScriptNodeKind::ElseStatement
    {
        render_elements.push(RenderElement::HardLine);
        return;
    }

    if current_kind == GDScriptNodeKind::Body
        || current_kind == GDScriptNodeKind::ClassBody
        || current_kind == GDScriptNodeKind::MatchBody
        || current_kind == GDScriptNodeKind::SetBody
        || current_kind == GDScriptNodeKind::GetBody
    {
        render_elements.push(RenderElement::HardLine);
        return;
    }

    if current_kind == GDScriptNodeKind::SetGet {
        return;
    }

    if previous_kind == GDScriptNodeKind::TokenParen
        || previous_kind == GDScriptNodeKind::TokenBracket
        || previous_kind == GDScriptNodeKind::TokenBrace
    {
        return;
    }

    if previous_kind == GDScriptNodeKind::Comment {
        render_elements.push(RenderElement::HardLine);
        return;
    }

    if current_kind == GDScriptNodeKind::TokenParen
        || current_kind == GDScriptNodeKind::TokenBracket
        || current_kind == GDScriptNodeKind::TokenBrace
    {
        return;
    }

    if previous_kind == GDScriptNodeKind::TokenDot || current_kind == GDScriptNodeKind::TokenDot {
        return;
    }

    if current_kind == GDScriptNodeKind::TokenComma {
        return;
    }

    if current_kind == GDScriptNodeKind::TokenColon {
        return;
    }

    if (previous_kind == GDScriptNodeKind::Identifier
        || previous_kind == GDScriptNodeKind::NameInit
        || previous_kind == GDScriptNodeKind::NameSet
        || previous_kind == GDScriptNodeKind::NameGet)
        && (current_kind == GDScriptNodeKind::Parameters
            || current_kind == GDScriptNodeKind::Arguments
            || current_kind == GDScriptNodeKind::SubscriptArguments)
    {
        return;
    }

    if previous_kind == GDScriptNodeKind::LineContinuation {
        return;
    }

    if parent_kind == GDScriptNodeKind::UnaryOperator
        && (previous_child.kind() == "~" || previous_kind == GDScriptNodeKind::Operator)
    {
        return;
    }

    if parent_kind == GDScriptNodeKind::Lambda
        && previous_kind == GDScriptNodeKind::KeywordFunc
        && current_kind == GDScriptNodeKind::Parameters
    {
        return;
    }

    if previous_child.kind() == "@" {
        return;
    }

    // If the previous child is an annotations node and any of the annotations
    // has arguments (e.g. @rpc("any_peer")), keep the annotation on its own line.
    if previous_kind == GDScriptNodeKind::Annotations {
        let annotation_count = previous_child.child_count();
        let mut annotation_index = 0;
        while annotation_index < annotation_count {
            if let Some(annotation) = previous_child.child(annotation_index as u32) {
                if annotation.child_count() > 2 {
                    render_elements.push(RenderElement::HardLine);
                    return;
                }
            }
            annotation_index += 1;
        }
    }

    render_elements.push(RenderElement::Space);
}

/// Formats the source node (topmost tree-sitter AST node) when the reorder_code
/// option is enabled. Calls the code sorting module to sort top-level
/// declarations into groups (signals, enums, constants, variables, methods,
/// classes).
fn process_source_reorder(
    input: &ParseInput,
    node: tree_sitter::Node,
    render_elements: &mut Vec<RenderElement>,
) {
    let source = input.source;
    let plan = reorder::build_reorder_plan(node, source);
    let mut previous_classification: Option<DeclarationKind> = None;
    let mut previous_is_double_spaced = false;
    let mut is_first = true;

    let mut item_index = 0;
    while item_index < plan.items.len() {
        let item = &plan.items[item_index];
        let current_needs_two_blank = matches!(
            item.classification,
            DeclarationKind::Method | DeclarationKind::InnerClass
        );
        if !is_first {
            if let Some(ref previous_child) = previous_classification {
                if previous_child == &item.classification
                    && !previous_is_double_spaced
                    && !current_needs_two_blank
                {
                    render_elements.push(RenderElement::HardLine);
                } else if previous_is_double_spaced || current_needs_two_blank {
                    let count = if current_needs_two_blank {
                        input.blank_lines_around_definitions
                    } else {
                        2
                    };
                    push_blank_lines(render_elements, count);
                } else if matches!(
                    previous_child,
                    DeclarationKind::ClassAnnotation
                        | DeclarationKind::ClassName
                        | DeclarationKind::Extends
                        | DeclarationKind::Docstring
                ) && matches!(
                    item.classification,
                    DeclarationKind::ClassAnnotation
                        | DeclarationKind::ClassName
                        | DeclarationKind::Extends
                        | DeclarationKind::Docstring
                ) {
                    render_elements.push(RenderElement::HardLine);
                } else {
                    render_elements.push(RenderElement::HardLine);
                    render_elements.push(RenderElement::BlankLine);
                }
            }
        }
        is_first = false;
        previous_classification = Some(item.classification);
        previous_is_double_spaced = current_needs_two_blank;

        // output leading comments/annotations.
        if item.classification != DeclarationKind::Docstring {
            let declaration_start = match node.child(item.child_index as u32) {
                Some(child) => child.start_byte(),
                None => 0,
            };
            let mut leading_index = 0;
            while leading_index < item.leading_indices.len() {
                let leading_child_index = item.leading_indices[leading_index];
                if let Some(child) = node.child(leading_child_index as u32) {
                    process_node(input, child, render_elements);
                    let next_start = if leading_index + 1 < item.leading_indices.len() {
                        let lookahead_index = item.leading_indices[leading_index + 1];
                        match node.child(lookahead_index as u32) {
                            Some(next_child) => next_child.start_byte(),
                            None => declaration_start,
                        }
                    } else {
                        declaration_start
                    };
                    if has_newline(source, child.end_byte(), next_start) {
                        render_elements.push(RenderElement::HardLine);
                    } else {
                        render_elements.push(RenderElement::Space);
                    }
                }
                leading_index += 1;
            }
        }

        if item.classification == DeclarationKind::Docstring {
            let mut docstring_index = 0;
            while docstring_index < item.leading_indices.len() {
                let docstring_child_index = item.leading_indices[docstring_index];
                if let Some(child) = node.child(docstring_child_index as u32) {
                    process_node(input, child, render_elements);
                    render_elements.push(RenderElement::HardLine);
                }
                docstring_index += 1;
            }
        } else if let Some(child) = node.child(item.child_index as u32) {
            if let Some(sub_child_index) = item.sub_child {
                if let Some(sub) = child.child(sub_child_index as u32) {
                    process_node(input, sub, render_elements);
                }
            } else if item.split_extends {
                // When splitting extends, we build the class_name_statement children
                // one by one, skipping the extends child (it's emitted as a separate
                // reorder item).
                let parent_kind = GDScriptNodeKind::get_kind_from_ast_node(child);
                let class_name_child_count = child.child_count();
                let mut previous_node: Option<tree_sitter::Node> = None;
                let mut child_index: usize = 0;
                while child_index < class_name_child_count {
                    let Some(sub) = child.child(child_index as u32) else {
                        child_index += 1;
                        continue;
                    };
                    let sub_child_kind = GDScriptNodeKind::get_kind_from_ast_node(sub);
                    if sub_child_kind == GDScriptNodeKind::Extends {
                        child_index += 1;
                        continue;
                    }
                    if let Some(ref previous_node) = previous_node {
                        emit_inter_child_separator(
                            parent_kind,
                            previous_node,
                            &sub,
                            render_elements,
                        );
                    }
                    process_node(input, sub, render_elements);
                    previous_node = Some(sub);
                    child_index += 1;
                }
            } else {
                process_node(input, child, render_elements);
            }
        }

        let declaration_end: usize = match node.child(item.child_index as u32) {
            Some(child) => child.end_byte(),
            None => 0,
        };
        let mut trailing_index = 0;
        while trailing_index < item.trailing_indices.len() {
            let trailing_child_index = item.trailing_indices[trailing_index];
            if let Some(child) = node.child(trailing_child_index as u32) {
                if has_newline(source, declaration_end, child.start_byte()) {
                    render_elements.push(RenderElement::HardLine);
                } else {
                    render_elements.push(RenderElement::Space);
                }
                process_node(input, child, render_elements);
            }
            trailing_index += 1;
        }
        item_index += 1;
    }
}

/// Entry point for formatting a code file. Takes the root parsed GDScript AST
/// node and starts walking through the AST. This function directly populates
/// the `render_elements` argument passed in with tokens that form an intermediate
/// representation of the code that the renderer uses to build the final output.
///
/// We use a Wadler-style pretty printing algorithm to build the formatted
/// output in the renderer module. This formatter module places potential line
/// breaks in the `render_elements` vector, and the renderer then decides where
/// to apply line breaks depending on the desired (and configurable) maximum
/// line length.
pub fn build_formatter_intermediate_representation(
    input: &ParseInput,
    render_elements: &mut Vec<RenderElement>,
) {
    render_elements.clear();
    let root = input.tree.root_node();
    render_elements.reserve(root.named_child_count() * 8);
    process_source(input, root, render_elements);
}
