//! This module exposes a function that reorders GDScript code according to the
//! official GDScript style guide.
//!
//! It works as a separate processing pass that parses the GDScript code using
//! tree-sitter, detects top-level declarations, and reorders them according to
//! the style guide.
//!
//! We assume that you won't run this on every save, but rather manually using
//! a code editor command or task when you're met with a messy file.
use tree_sitter::{Node, Query, QueryCursor, StreamingIterator, Tree};

/// This method parses the GDScript content, extracts top-level elements,
/// and reorders them according to the GDScript style guide.
pub fn reorder_gdscript_elements(
    tree: &Tree,
    content: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let tokens = extract_tokens_to_reorder(&tree, content)?;
    let ordered_elements = sort_gdscript_tokens(tokens);
    let reordered_content = build_reordered_code(ordered_elements, content);

    Ok(reordered_content)
}

/// This struct is used to hold an element along with its associated comments
/// and original text so we can precisely reconstruct it, and also when we move
/// functions etc. their docstrings and comments come along.
#[derive(Debug, Clone)]
pub struct GDScriptTokensWithComments {
    pub token_kind: GDScriptTokenKind,
    pub attached_comments: Vec<String>,
    pub trailing_comments: Vec<String>,
    pub original_text: String,
    pub start_byte: usize,
    pub end_byte: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GDScriptTokenKind {
    ClassAnnotation(String), // Annotations that go at the top of the file like @tool and @icon
    ClassName(String),       // This is the class_name declaration
    Extends(String),         // extends keyword and its argument
    Docstring(String), // Represents docstrings, commentsa that are above a declaration and start with ##
    Signal(String, bool), // Represents a signal. The second value indicates if it's pseudo-private (starts with _)
    // All the following types have the second bool value indicating if it's pseudo-private
    Enum(String, bool),
    Constant(String, bool),
    StaticVariable(String, bool),
    ExportVariable(String, bool),
    RegularVariable(String, bool),
    OnReadyVariable(String, bool),
    // For methods we also store their kind (static function, built-in virtual method overriden from Godot, or "custom")
    Method(String, MethodType, bool),
    InnerClass(String, bool),
    // This is for cases like new syntax as it comes out - in general, elements
    // we don't recognize and we don't want to mess up
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub enum MethodType {
    // This is a special case for _static_init()
    StaticInit,
    StaticFunction,
    // This is for built-in virtual methods like _init(), _ready(), _process(), etc.
    BuiltinVirtual(u8),
    // This is for all other methods defined by the user
    Custom,
}

/// This represents a parsed tree-sitter node that we've classified to see if it's something we can reorder.
///
/// When we go through the GDScript file, we look at each piece of code (like functions, variables, comments)
/// and figure out what it is. This struct holds that information plus the original node and text.
///
/// The `reorderable_element` field tells us if this piece of code is something we know how to reorder
/// (like a function or variable) or if it's something we should just leave alone (like a random comment).
#[derive(Debug, Clone)]
struct ClassifiedElement<'a> {
    /// Reference to the original tree-sitter node from parsing the file
    node: tree_sitter::Node<'a>,
    /// The text content of this piece of code
    text: String,
    /// If we can reorder this element, this contains tells us the node kind
    /// (method, member variable, etc.). If we don't know what it is (e.g. new
    /// syntax we don't support yet in new Godot version) or can't reorder it,
    /// this is None.
    reorderable_element: Option<GDScriptTokenKind>,
}

/// This constant lists built-in virtual methods in the order they should appear.
/// The higher the method is in the list, the higher the priority (i.e. _init comes before _ready).
const BUILTIN_VIRTUAL_METHODS: &[&str] = &[
    "_init",
    "_enter_tree",
    "_ready",
    "_process",
    "_physics_process",
    "_exit_tree",
    "_input",
    "_unhandled_input",
    "_gui_input",
    "_draw",
    "_notification",
    "_get_configuration_warnings",
    "_validate_property",
    "_get_property_list",
    "_property_can_revert",
    "_property_get_revert",
    "_get",
    "_set",
    "_to_string",
];

impl GDScriptTokenKind {
    /// Returns the ordering priority for this kind of declaration. The lower the
    /// number, the higher the priority.
    pub fn get_priority(&self) -> u8 {
        match self {
            GDScriptTokenKind::ClassAnnotation(_) => 1,
            GDScriptTokenKind::ClassName(_) => 2,
            GDScriptTokenKind::Extends(_) => 3,
            GDScriptTokenKind::Docstring(_) => 4,
            GDScriptTokenKind::Signal(_, _) => 5,
            GDScriptTokenKind::Enum(_, _) => 6,
            GDScriptTokenKind::Constant(_, _) => 7,
            GDScriptTokenKind::StaticVariable(_, _) => 8,
            GDScriptTokenKind::ExportVariable(_, _) => 9,
            GDScriptTokenKind::RegularVariable(_, _) => 10,
            GDScriptTokenKind::OnReadyVariable(_, _) => 11,
            GDScriptTokenKind::Method(_, MethodType::StaticInit, _) => 12,
            GDScriptTokenKind::Method(_, MethodType::StaticFunction, _) => 13,
            GDScriptTokenKind::Method(_, MethodType::BuiltinVirtual(_), _) => 14,
            GDScriptTokenKind::Method(_, MethodType::Custom, _) => 15,
            GDScriptTokenKind::InnerClass(_, _) => 16,
            GDScriptTokenKind::Unknown(_) => 255,
        }
    }

    /// Returns the name of the element. This is used to sort elements of the
    /// same type alphabetically.
    pub fn get_name(&self) -> &str {
        match self {
            GDScriptTokenKind::ClassAnnotation(name) => name,
            GDScriptTokenKind::ClassName(name) => name,
            GDScriptTokenKind::Extends(name) => name,
            GDScriptTokenKind::Docstring(name) => name,
            GDScriptTokenKind::Signal(name, _) => name,
            GDScriptTokenKind::Enum(name, _) => name,
            GDScriptTokenKind::Constant(name, _) => name,
            GDScriptTokenKind::StaticVariable(name, _) => name,
            GDScriptTokenKind::ExportVariable(name, _) => name,
            GDScriptTokenKind::RegularVariable(name, _) => name,
            GDScriptTokenKind::OnReadyVariable(name, _) => name,
            GDScriptTokenKind::Method(name, _, _) => name,
            GDScriptTokenKind::InnerClass(name, _) => name,
            GDScriptTokenKind::Unknown(name) => name,
        }
    }

    /// Returns whether this element is private (starts with underscore).
    pub fn is_private(&self) -> bool {
        match self {
            GDScriptTokenKind::Signal(_, is_private) => *is_private,
            GDScriptTokenKind::Enum(_, is_private) => *is_private,
            GDScriptTokenKind::Constant(_, is_private) => *is_private,
            GDScriptTokenKind::StaticVariable(_, is_private) => *is_private,
            GDScriptTokenKind::ExportVariable(_, is_private) => *is_private,
            GDScriptTokenKind::RegularVariable(_, is_private) => *is_private,
            GDScriptTokenKind::OnReadyVariable(_, is_private) => *is_private,
            GDScriptTokenKind::Method(_, _, is_private) => *is_private,
            GDScriptTokenKind::InnerClass(_, is_private) => *is_private,
            _ => false,
        }
    }
}

/// This enum is used to group elements into broader categories to determine
/// how much spacing to add between them when rebuilding the code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokenKind {
    // This is for the top of the class (@tool, class name etc)
    Header,
    Signal,
    Enum,
    Constant,
    StaticVariable,
    ExportVariable,
    RegularVariable,
    OnReadyVariable,
    Method,
    InnerClass,
}

/// Gets the element type for grouping purposes.
fn get_token_kind(token_kind: &GDScriptTokenKind) -> TokenKind {
    match token_kind {
        GDScriptTokenKind::ClassAnnotation(_) => TokenKind::Header,
        GDScriptTokenKind::ClassName(_) => TokenKind::Header,
        GDScriptTokenKind::Extends(_) => TokenKind::Header,
        GDScriptTokenKind::Docstring(_) => TokenKind::Header,
        GDScriptTokenKind::Signal(_, _) => TokenKind::Signal,
        GDScriptTokenKind::Enum(_, _) => TokenKind::Enum,
        GDScriptTokenKind::Constant(_, _) => TokenKind::Constant,
        GDScriptTokenKind::StaticVariable(_, _) => TokenKind::StaticVariable,
        GDScriptTokenKind::ExportVariable(_, _) => TokenKind::ExportVariable,
        GDScriptTokenKind::RegularVariable(_, _) => TokenKind::RegularVariable,
        GDScriptTokenKind::OnReadyVariable(_, _) => TokenKind::OnReadyVariable,
        GDScriptTokenKind::Method(_, _, _) => TokenKind::Method,
        GDScriptTokenKind::InnerClass(_, _) => TokenKind::InnerClass,
        GDScriptTokenKind::Unknown(_) => TokenKind::Method,
    }
}

/// Extracts all top-level elements from the parsed tree.
fn extract_tokens_to_reorder(
    tree: &Tree,
    content: &str,
) -> Result<Vec<GDScriptTokensWithComments>, Box<dyn std::error::Error>> {
    let root = tree.root_node();
    let mut elements = Vec::new();

    // This query covers all top-level elements (direct children of source)
    // We need to capture everything so nothing gets lost
    let query_str = r#"
        (source (_) @element)
    "#;

    let query = Query::new(&tree_sitter_gdscript::LANGUAGE.into(), query_str)?;
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, root, content.as_bytes());

    let mut all_nodes = Vec::new();

    // Collect all the nodes and their positions
    while let Some(m) = matches.next() {
        for capture in m.captures {
            let node = capture.node;
            let text = node.utf8_text(content.as_bytes())?;
            all_nodes.push((node, text.to_string()));
        }
    }

    // First we process the top of the node tree. We look for the class docstring.
    // For now we treat them as any ## comments that appear before any declaration
    // like a variable or function. We collect them and then attach them to the
    // extends statement if we find one.
    //
    // TODO: Nathan (GDQuest): this is not perfect, we need to handle more edge cases, but I'm
    // pushing this for now to make the command more usable. We can improve this later.
    // Notably a comment after the extends declaration might be a var or method docstring.
    // We need to check if the comments are contiguous with the declaration they are
    // attached to.
    let mut class_docstring_comments = Vec::new();
    let mut found_non_comment_non_class = false;
    for (node, text) in &all_nodes {
        match node.kind() {
            "comment" => {
                if text.trim_start().starts_with("##") && !found_non_comment_non_class {
                    class_docstring_comments.push(text.clone());
                }
            }
            "class_name_statement" | "extends_statement" | "annotation" => {
                continue;
            }
            // Any other element means we're past the top of the file, so we stop
            // collecting the class docstring
            _ => {
                found_non_comment_non_class = true;
            }
        }
    }

    let mut classified_elements = Vec::new();
    // Here we associate comments and annotations with the next declaration. We
    // loop through the node tree from top to bottom, collecting comments and
    // annotations until we hit a declaration, at which point we attach the
    // collected comments/annotations to that declaration.
    for (node, text) in &all_nodes {
        let reorderable_element = classify_element(*node, &text, content)?;
        classified_elements.push(ClassifiedElement {
            node: *node,
            text: text.clone(),
            reorderable_element,
        });
    }
    let mut pending_comments = Vec::new();
    let mut pending_annotations = Vec::new();
    let mut found_extends_declaration = false;
    let mut class_docstring_attached = false;
    // TODO: Handle multiple #region/#endregion pairs properly
    // Nathan: For now we just attach the last #endregion to the most recent function
    // that has a #region comment, to handle the most common use case
    // Regions generally are tricky to reorder as they can span multiple
    // functions that should be reordered. In those cases I would recommend users not to
    // use regions though, or not to use the reorder feature
    let mut region_end_comment = None;

    for classified in classified_elements {
        let node = classified.node;
        let text = classified.text;
        let reorderable_element = classified.reorderable_element;
        match node.kind() {
            "comment" => {
                // We already processed class docstring comments, so we skip them here
                // This may look inefficient but in practice it should not have much impact
                if text.trim_start().starts_with("##") && class_docstring_comments.contains(&text) {
                    continue;
                } else {
                    pending_comments.push(text);
                }
            }
            "region_start" => {
                pending_comments.push(text);
            }
            "region_end" => {
                region_end_comment = Some(text.clone());
            }
            "annotation" => {
                if let Some(element) = reorderable_element {
                    match element {
                        GDScriptTokenKind::ClassAnnotation(_) => {
                            elements.push(GDScriptTokensWithComments {
                                token_kind: element,
                                attached_comments: Vec::new(),
                                trailing_comments: Vec::new(),
                                original_text: text,
                                start_byte: node.start_byte(),
                                end_byte: node.end_byte(),
                            });
                        }
                        _ => {
                            pending_annotations.push(text);
                        }
                    }
                } else {
                    pending_annotations.push(text);
                }
            }
            "class_name_statement" => {
                if let Some(element) = reorderable_element {
                    // Don't attach class docstring to class_name, save it for extends
                    let mut non_docstring_comments = Vec::new();
                    for comment in &pending_comments {
                        if !class_docstring_comments.contains(comment) {
                            non_docstring_comments.push(comment.clone());
                        }
                    }
                    elements.push(GDScriptTokensWithComments {
                        token_kind: element,
                        attached_comments: non_docstring_comments,
                        trailing_comments: Vec::new(),
                        original_text: text,
                        start_byte: node.start_byte(),
                        end_byte: node.end_byte(),
                    });
                    pending_comments.clear();
                    pending_annotations.clear();
                }
            }
            "extends_statement" => {
                found_extends_declaration = true;
                if let Some(element) = reorderable_element {
                    elements.push(GDScriptTokensWithComments {
                        token_kind: element,
                        attached_comments: pending_comments.clone(),
                        trailing_comments: Vec::new(),
                        original_text: text,
                        start_byte: node.start_byte(),
                        end_byte: node.end_byte(),
                    });
                    pending_comments.clear();
                    pending_annotations.clear();

                    // Create separate docstring element if we have class docstrings
                    if !class_docstring_attached && !class_docstring_comments.is_empty() {
                        let docstring_text = class_docstring_comments.join("\n");
                        elements.push(GDScriptTokensWithComments {
                            token_kind: GDScriptTokenKind::Docstring(docstring_text.clone()),
                            attached_comments: Vec::new(),
                            trailing_comments: Vec::new(),
                            original_text: docstring_text,
                            start_byte: 0,
                            end_byte: 0,
                        });
                        class_docstring_attached = true;
                    }
                }
            }
            _ => {
                if let Some(element) = reorderable_element {
                    // If we haven't attached class docstring yet and this is the first real element,
                    // create a separate docstring element (for cases where there's no extends)
                    if !class_docstring_attached
                        && !class_docstring_comments.is_empty()
                        && !found_extends_declaration
                    {
                        let docstring_text = class_docstring_comments.join("\n");
                        elements.push(GDScriptTokensWithComments {
                            token_kind: GDScriptTokenKind::Docstring(docstring_text.clone()),
                            attached_comments: Vec::new(),
                            trailing_comments: Vec::new(),
                            original_text: docstring_text,
                            start_byte: 0,
                            end_byte: 0,
                        });
                        class_docstring_attached = true;
                    }

                    let mut combined_comments = pending_annotations.clone();
                    combined_comments.extend(pending_comments.clone());

                    // We store trailing #endregion comments to attach them to
                    // the most recent function that has a #region comment at
                    // the top, to move them along with the function when
                    // reordering
                    if let Some(region_end) = region_end_comment.take() {
                        for i in (0..elements.len()).rev() {
                            if matches!(elements[i].token_kind, GDScriptTokenKind::Method(_, _, _))
                            {
                                let has_region = elements[i]
                                    .attached_comments
                                    .iter()
                                    .any(|c| c.trim().starts_with("#region"));
                                if has_region {
                                    elements[i].trailing_comments.push(region_end.clone());
                                    break;
                                }
                            }
                        }
                    }

                    elements.push(GDScriptTokensWithComments {
                        token_kind: element,
                        attached_comments: combined_comments,
                        trailing_comments: Vec::new(),
                        original_text: text,
                        start_byte: node.start_byte(),
                        end_byte: node.end_byte(),
                    });
                    pending_comments.clear();
                    pending_annotations.clear();
                } else {
                    // We create unknown element for unhandled nodes to preserve
                    // them. Given how the module works, if we don't do that the
                    // nodes will be dropped.
                    elements.push(GDScriptTokensWithComments {
                        token_kind: GDScriptTokenKind::Unknown(text.clone()),
                        attached_comments: pending_comments.clone(),
                        trailing_comments: Vec::new(),
                        original_text: text,
                        start_byte: node.start_byte(),
                        end_byte: node.end_byte(),
                    });
                    pending_comments.clear();
                    pending_annotations.clear();
                }
            }
        }
    }

    Ok(elements)
}

/// This function classifies a parsed tree sitter node into a GDScriptElement.
fn classify_element(
    node: Node,
    text: &str,
    content: &str,
) -> Result<Option<GDScriptTokenKind>, Box<dyn std::error::Error>> {
    match node.kind() {
        "annotation" => {
            if text.starts_with("@tool")
                || text.starts_with("@icon")
                || text.starts_with("@static_unload")
            {
                Ok(Some(GDScriptTokenKind::ClassAnnotation(text.to_string())))
            } else {
                Ok(None)
            }
        }
        "class_name_statement" => {
            // If the class_name statement also has an extends in it, we split
            // it into two separate elements on two lines.
            if text.contains("extends") {
                let parts: Vec<&str> = text.splitn(2, "extends").collect();
                if parts.len() == 2 {
                    // We'll handle this case in the extraction logic
                    Ok(Some(GDScriptTokenKind::ClassName(
                        parts[0].trim().to_string(),
                    )))
                } else {
                    Ok(Some(GDScriptTokenKind::ClassName(text.to_string())))
                }
            } else {
                Ok(Some(GDScriptTokenKind::ClassName(text.to_string())))
            }
        }
        "extends_statement" => Ok(Some(GDScriptTokenKind::Extends(text.to_string()))),
        "comment" => Ok(None),
        "region_start" => Ok(None),
        "region_end" => Ok(None),
        "signal_statement" => {
            let name = extract_signal_name(node, content)?;
            let is_private = name.starts_with('_');
            Ok(Some(GDScriptTokenKind::Signal(name, is_private)))
        }
        "enum_definition" => {
            let name = extract_enum_name(node, content)?;
            let is_private = name.starts_with('_');
            Ok(Some(GDScriptTokenKind::Enum(name, is_private)))
        }
        "const_statement" => {
            let name = extract_const_name(node, content)?;
            let is_private = name.starts_with('_');
            Ok(Some(GDScriptTokenKind::Constant(name, is_private)))
        }
        "variable_statement" => classify_variable_statement(node, content),
        "function_definition" | "constructor_definition" => {
            let name = extract_function_name(node, content)?;
            let is_static = is_static_method(node, content);
            let is_private = name.starts_with('_');

            let method_type = if name == "_static_init" {
                MethodType::StaticInit
            } else if is_static {
                MethodType::StaticFunction
            } else if let Some(priority) = get_builtin_virtual_priority(&name) {
                MethodType::BuiltinVirtual(priority)
            } else {
                MethodType::Custom
            };

            Ok(Some(GDScriptTokenKind::Method(
                name,
                method_type,
                is_private,
            )))
        }
        "class_definition" => {
            let name = extract_class_name(node, content)?;
            let is_private = name.starts_with('_');
            Ok(Some(GDScriptTokenKind::InnerClass(name, is_private)))
        }
        _ => Ok(Some(GDScriptTokenKind::Unknown(text.to_string()))),
    }
}

/// This function classifies a variable statement into the correct variable type to figure out how to order it.
fn classify_variable_statement(
    node: Node,
    content: &str,
) -> Result<Option<GDScriptTokenKind>, Box<dyn std::error::Error>> {
    let text = node.utf8_text(content.as_bytes())?;
    let variable_name = extract_variable_name(node, content)?;
    let is_private = variable_name.starts_with('_');

    // Look for annotations in the node's text string, which we use to sort the
    // variables
    let has_export = text.contains("@export");
    let has_onready = text.contains("@onready");
    let has_static = text.contains("static var");

    if has_export {
        Ok(Some(GDScriptTokenKind::ExportVariable(
            variable_name,
            is_private,
        )))
    } else if has_onready {
        Ok(Some(GDScriptTokenKind::OnReadyVariable(
            variable_name,
            is_private,
        )))
    } else if has_static {
        Ok(Some(GDScriptTokenKind::StaticVariable(
            variable_name,
            is_private,
        )))
    } else {
        Ok(Some(GDScriptTokenKind::RegularVariable(
            variable_name,
            is_private,
        )))
    }
}

/// Returns the name of the signal from a signal statement node.
fn extract_signal_name(node: Node, content: &str) -> Result<String, Box<dyn std::error::Error>> {
    let text = node.utf8_text(content.as_bytes())?;
    let Some(name) = text.strip_prefix("signal ") else {
        return Ok("unknown_signal".to_string());
    };

    if let Some((name, _)) = name.split_once(|c: char| c == '(' || c == ':' || c.is_whitespace()) {
        return Ok(name.to_string());
    }

    Ok(name.to_string())
}

/// Returns the name of the enum from an enum definition node.
fn extract_enum_name(node: Node, content: &str) -> Result<String, Box<dyn std::error::Error>> {
    let text = node.utf8_text(content.as_bytes())?;
    let Some(name) = text.strip_prefix("enum ") else {
        return Ok("unknown_enum".to_string());
    };

    if let Some(name) = name
        .split_once(|c: char| c == '{' || c.is_whitespace())
        .map(|(n, _)| n.trim())
        && !name.is_empty()
    {
        Ok(name.to_string())
    } else {
        Ok("unnamed_enum".to_string())
    }
}

/// Returns the name of the constant from a const statement node.
fn extract_const_name(node: Node, content: &str) -> Result<String, Box<dyn std::error::Error>> {
    let text = node.utf8_text(content.as_bytes())?;
    let Some(name) = text.strip_prefix("const ") else {
        return Ok("unknown_const".to_string());
    };

    if let Some((name, _)) = name.split_once(|c: char| c == '=' || c == ':' || c.is_whitespace()) {
        return Ok(name.trim().to_string());
    }

    Ok(name.trim().to_string())
}

/// Returns the name of the variable from a var statement node.
fn extract_variable_name(node: Node, content: &str) -> Result<String, Box<dyn std::error::Error>> {
    let text = node.utf8_text(content.as_bytes())?;

    let Some(name) = text.strip_prefix("var ") else {
        return Ok("unknown_var".to_string());
    };

    if let Some((name, _)) = name.split_once(|c: char| c == ':' || c == '=' || c.is_whitespace()) {
        return Ok(name.trim().to_string());
    }

    Ok(name.trim().to_string())
}

/// Returns the name of the function from a function definition node.
fn extract_function_name(node: Node, content: &str) -> Result<String, Box<dyn std::error::Error>> {
    let text = node.utf8_text(content.as_bytes())?;

    let Some(name) = text.strip_prefix("func ") else {
        return Ok("unknown_func".to_string());
    };

    if let Some((name, _)) = name.split_once('(') {
        Ok(name.trim().to_string())
    } else {
        Ok("unknown_func".to_string())
    }
}

/// Returns the name of an inner class from a class definition node.
fn extract_class_name(node: Node, content: &str) -> Result<String, Box<dyn std::error::Error>> {
    let text = node.utf8_text(content.as_bytes())?;
    let Some(name) = text.strip_prefix("class ") else {
        return Ok("unknown_class".to_string());
    };

    if let Some((name, _)) = name.split_once(':') {
        return Ok(name.trim().to_string());
    }

    Ok(name.trim().to_string())
}

fn is_static_method(node: Node, content: &str) -> bool {
    let text = node.utf8_text(content.as_bytes()).unwrap_or("");
    text.contains("static func")
}

fn get_builtin_virtual_priority(method_name: &str) -> Option<u8> {
    BUILTIN_VIRTUAL_METHODS
        .iter()
        .enumerate()
        // Position in the list is the priority
        .find_map(|(index, name)| (*name == method_name).then_some((index + 1) as u8))
}

/// Sorts declarations according to the GDScript style guide and returns the ordered list.
fn sort_gdscript_tokens(
    mut tokens: Vec<GDScriptTokensWithComments>,
) -> Vec<GDScriptTokensWithComments> {
    tokens.sort_by(|a, b| {
        let priority_cmp = a
            .token_kind
            .get_priority()
            .cmp(&b.token_kind.get_priority());
        if priority_cmp != std::cmp::Ordering::Equal {
            return priority_cmp;
        }

        // For methods, we sort by method type
        if let (GDScriptTokenKind::Method(_, type_a, _), GDScriptTokenKind::Method(_, type_b, _)) =
            (&a.token_kind, &b.token_kind)
        {
            let type_cmp = type_a.cmp(type_b);
            if type_cmp != std::cmp::Ordering::Equal {
                return type_cmp;
            }

            // For built-in virtual methods, we sort them by our priority list
            if let (MethodType::BuiltinVirtual(p_a), MethodType::BuiltinVirtual(p_b)) =
                (type_a, type_b)
            {
                let builtin_cmp = p_a.cmp(p_b);
                if builtin_cmp != std::cmp::Ordering::Equal {
                    return builtin_cmp;
                }
            }
        }

        // Third, sort public before pseudo-private declarations
        let privacy_cmp = a.token_kind.is_private().cmp(&b.token_kind.is_private());
        if privacy_cmp != std::cmp::Ordering::Equal {
            return privacy_cmp;
        }

        // Finally we sort alphabetically. We also handle the top annotations up here.
        match (&a.token_kind, &b.token_kind) {
            (
                GDScriptTokenKind::ClassAnnotation(a_text),
                GDScriptTokenKind::ClassAnnotation(b_text),
            ) => {
                // @tool should generally be at the very top of the script so we give it top priority
                let a_priority = if a_text.starts_with("@tool") {
                    0
                } else if a_text.starts_with("@icon") {
                    1
                } else {
                    2
                };
                let b_priority = if b_text.starts_with("@tool") {
                    0
                } else if b_text.starts_with("@icon") {
                    1
                } else {
                    2
                };
                a_priority.cmp(&b_priority)
            }
            _ => a.token_kind.get_name().cmp(b.token_kind.get_name()),
        }
    });

    tokens
}

/// This function takes the sorted declarations/code elements and rebuilds the
/// GDScript code string from them.
fn build_reordered_code(
    tokens: Vec<GDScriptTokensWithComments>,
    _original_content: &str,
) -> String {
    let mut output = String::new();
    let mut previous_token_kind = None;

    for current_token in tokens {
        let current_token_type = get_token_kind(&current_token.token_kind);
        let is_function = matches!(current_token.token_kind, GDScriptTokenKind::Method(_, _, _));

        let is_inner_class = matches!(
            current_token.token_kind,
            GDScriptTokenKind::InnerClass(_, _)
        );
        // If true, we need to add spacing before this element, either single or
        // double line breaks depending on the context.
        let needs_spacing = if output.is_empty() {
            false
        } else if let Some(previous_kind) = previous_token_kind {
            if previous_kind != current_token_type {
                // We're leaving one group of tokens for another (like previous
                // was variables, now we're seeing a function) -> needs spacing
                true
            } else if is_function {
                // Between functions we always want two line breaks
                true
            } else if is_inner_class && previous_kind == TokenKind::InnerClass {
                // Between inner classes, same as functions
                true
            } else {
                // If we reach here we're seeing the same kind of token as
                // before, like two regular variables in a row or two signals in
                // a row - we don't need extra spacing
                false
            }
        } else {
            false
        };

        if needs_spacing {
            if is_function {
                output.push_str("\n\n");
            } else if is_inner_class && previous_token_kind == Some(TokenKind::Method) {
                output.push_str("\n\n");
            } else if is_inner_class && previous_token_kind == Some(TokenKind::InnerClass) {
                output.push_str("\n\n");
            } else {
                output.push('\n');
            }
        }

        // Check and add any comments that were found right before this element
        // in the original code (like docstrings before a function)
        for comment in &current_token.attached_comments {
            output.push_str(comment);
            if !comment.ends_with('\n') {
                output.push('\n');
            }
        }
        // Insert the token's original text (function, variable, etc.)
        output.push_str(&current_token.original_text);
        if !current_token.original_text.ends_with('\n') {
            output.push('\n');
        }
        // After inserting the token, we also add any trailing comments that were
        // found right after it in the original code (like #endregion after a function)
        for comment in &current_token.trailing_comments {
            output.push_str(comment);
            if !comment.ends_with('\n') {
                output.push('\n');
            }
        }

        previous_token_kind = Some(current_token_type);
    }

    if !output.ends_with('\n') {
        output.push('\n');
    }

    output
}
