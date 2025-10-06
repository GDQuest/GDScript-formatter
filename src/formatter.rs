//! This module formats GDScript code using Topiary with tree-sitter to parse and
//! format GDScript files.
//!
//! After the main formatting pass through Topiary, we apply post-processing steps
//! to clean up and standardize the output. These include:
//!
//! - Adding vertical spacing between methods, classes, etc.
//! - Removing unnecessary blank lines that might have been added during formatting
//! - Removing dangling semicolons that sometimes end up on their own lines
//! - Cleaning up lines that contain only whitespace
//! - Optionally reordering code elements according to the GDScript style guide
//!
//! Some of the post-processing is outside of Topiary's capabilities, while other
//! rules have too much performance overhead when applied through Topiary.
use std::{collections::VecDeque, io::BufWriter};

use regex::{Regex, RegexBuilder, Replacer};
use topiary_core::{Language, Operation, TopiaryQuery, formatter_tree};
use tree_sitter::{Parser, Point, Query, QueryCursor, StreamingIterator, Tree};

use crate::FormatterConfig;

static QUERY: &str = include_str!("../queries/gdscript.scm");

pub fn format_gdscript(content: &str) -> Result<String, Box<dyn std::error::Error>> {
    format_gdscript_with_config(content, &FormatterConfig::default())
}

pub fn format_gdscript_with_config(
    content: &str,
    config: &FormatterConfig,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut formatter = Formatter::new(content.to_owned(), config.clone());

    formatter.preprocess().format()?.postprocess().reorder();
    formatter.finish()
}

struct Formatter {
    content: String,
    config: FormatterConfig,
    parser: Parser,
    input_tree: GdTree,
    tree: Tree,
}

impl Formatter {
    #[inline(always)]
    fn new(content: String, config: FormatterConfig) -> Self {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_gdscript::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(&content, None).unwrap();
        let mut input_tree = GdTree::from_ts_tree(&tree, content.as_bytes());
        input_tree.postprocess();

        Self {
            content,
            config,
            tree,
            input_tree,
            parser,
        }
    }

    #[inline(always)]
    fn format(&mut self) -> Result<&mut Self, Box<dyn std::error::Error>> {
        let indent_string = if self.config.use_spaces {
            " ".repeat(self.config.indent_size)
        } else {
            "\t".to_string()
        };

        let language = Language {
            name: "gdscript".to_owned(),
            query: TopiaryQuery::new(&tree_sitter_gdscript::LANGUAGE.into(), QUERY).unwrap(),
            grammar: tree_sitter_gdscript::LANGUAGE.into(),
            indent: Some(indent_string),
        };

        let mut output = Vec::new();
        let mut writer = BufWriter::new(&mut output);

        formatter_tree(
            self.tree.clone().into(),
            &self.content,
            &mut writer,
            &language,
            Operation::Format {
                skip_idempotence: true,
                tolerate_parsing_errors: true,
            },
        )
        .map_err(|e| format!("Topiary formatting failed: {e}"))?;

        drop(writer);

        self.content = String::from_utf8(output)
            .map_err(|e| format!("Failed to parse topiary output as UTF-8: {}", e))?;

        Ok(self)
    }

    #[inline(always)]
    fn reorder(&mut self) -> &mut Self {
        if !self.config.reorder_code {
            return self;
        }

        self.tree = self.parser.parse(&self.content, Some(&self.tree)).unwrap();
        match crate::reorder::reorder_gdscript_elements(&self.tree, &self.content) {
            Ok(reordered) => {
                self.content = reordered;
            }
            Err(e) => {
                eprintln!(
                    "Warning: Code reordering failed: {e}. Returning formatted code without reordering."
                );
            }
        };
        self
    }

    /// This function runs over the content before going through topiary.
    /// It is used to prepare the content for formatting or save performance by
    /// pre-applying rules that could be performance-intensive through topiary.
    #[inline(always)]
    fn preprocess(&mut self) -> &mut Self {
        self
    }

    /// This function runs over the content after going through topiary. We use it
    /// to clean up/balance out the output.
    #[inline(always)]
    fn postprocess(&mut self) -> &mut Self {
        self.add_newlines_after_extends_statement()
            .fix_dangling_semicolons()
            .fix_dangling_commas()
            .remove_trailing_commas_from_preload()
            .postprocess_tree_sitter()
    }

    /// Finishes formatting and returns the resulting file content.
    #[inline(always)]
    fn finish(mut self) -> Result<String, Box<dyn std::error::Error>> {
        if self.config.safe {
            self.tree = self.parser.parse(&self.content, None).unwrap();

            let output_tree = GdTree::from_ts_tree(&self.tree, self.content.as_bytes());
            if self.input_tree != output_tree {
                return Err("Code structure has changed after formatting".into());
            }
        }

        Ok(self.content)
    }

    /// This function adds additional new line characters after `extends_statement`.
    #[inline(always)]
    fn add_newlines_after_extends_statement(&mut self) -> &mut Self {
        // This regex matches substrings which:
        // - must start wtih "extends" keyword
        // - must contain `extends_name` character sequence that satisfies one of the following conditions:
        //   - consists out of alphanumeric characters
        //   - consists out of any characters (except new lines) between double quotes
        // - must contain at least one new line character between `extends_name` and optional doc comment
        // - may contain multiple doc comment lines that starts with `##` and ends with a new line character
        let re = RegexBuilder::new(
            r#"(?P<extends_line>^extends )(?P<extends_name>([a-zA-Z0-9]+|".*?"))\n+((?P<doc>(?:^##.*\n)+)(?:\z|\n))?\n*(?P<EOF>\z)?"#,
        )
        .multi_line(true)
        .build()
        .expect("regex should compile");

        self.regex_replace_all_outside_strings(re, |caps: &regex::Captures| {
            let extends_line = caps.name("extends_line").unwrap().as_str();
            let extends_name = caps.name("extends_name").unwrap().as_str();
            let doc = caps.name("doc").map(|m| m.as_str()).unwrap_or_default();
            // insert new line only if we are not at the end of file
            let blank_new_line = if caps.name("EOF").is_some() { "" } else { "\n" };

            format!(
                "{}{}\n{}{}",
                extends_line, extends_name, doc, blank_new_line
            )
        });

        self
    }

    /// This function fixes semicolons that end up on their own line with indentation
    /// by moving them to the end of the previous line.
    #[inline(always)]
    fn fix_dangling_semicolons(&mut self) -> &mut Self {
        if !self.content.contains(";") {
            return self;
        }
        let re_trailing = RegexBuilder::new(r"(\s*;)+$")
            .multi_line(true)
            .build()
            .expect("semicolon regex should compile");

        self.regex_replace_all_outside_strings(re_trailing, "");
        self
    }

    /// This function fixes commas that end up on their own line with indentation
    /// by moving them to the end of the previous line. This commonly happens
    /// with lambdas in data structures like arrays or function arguments.
    #[inline(always)]
    fn fix_dangling_commas(&mut self) -> &mut Self {
        // This targets cases where a comma is on its own line with only
        // whitespace before it instead of being at the end of the previous
        // line
        // Pattern: capture content before newline, then newline + whitespace + comma
        let re = RegexBuilder::new(r"([^\n\r])\n\s+,")
            .multi_line(true)
            .build()
            .expect("dangling comma regex should compile");

        self.regex_replace_all_outside_strings(re, "$1,");
        self
    }

    /// This function removes trailing commas from preload function calls.
    /// The GDScript parser doesn't support trailing commas in preload calls,
    /// but our formatter might add them for multi-line calls.
    #[inline(always)]
    fn remove_trailing_commas_from_preload(&mut self) -> &mut Self {
        let re = RegexBuilder::new(r"preload\s*\(([^)]*),(\s*)\)")
            .build()
            .expect("preload regex should compile");

        self.regex_replace_all_outside_strings(re, "preload($1$2)");
        self
    }

    /// This function runs postprocess passes that uses tree-sitter.
    #[inline(always)]
    fn postprocess_tree_sitter(&mut self) -> &mut Self {
        self.tree = self.parser.parse(&self.content, None).unwrap();

        self.handle_two_blank_line()
    }

    /// Replaces every match of regex `re` with `rep`, but only if the match is
    /// outside of strings (simple or multiline).
    /// Use this to make post-processing changes needed for formatting but that
    /// shouldn't affect strings in the source code.
    fn regex_replace_all_outside_strings<R: Replacer>(&mut self, re: Regex, mut rep: R) {
        let mut iter = re.captures_iter(&self.content).peekable();
        if iter.peek().is_none() {
            return;
        }

        let mut new = String::new();
        let mut last_match = 0;
        let mut start_position = Point::new(0, 0);

        // We first collect tree edits and then apply them, because regex returns positions from unmodified content
        let mut edits = Vec::new();

        for capture in iter {
            let m = capture.get(0).unwrap();
            let start_byte = m.start();
            let old_end_byte = m.end();
            let node = self
                .tree
                .root_node()
                .descendant_for_byte_range(start_byte, start_byte)
                .unwrap();
            if node.kind() == "string" {
                continue;
            }

            let mut replacement = String::new();
            rep.replace_append(&capture, &mut replacement);

            let new_end_byte = start_byte + replacement.len();

            let slice = &self.content[last_match..start_byte];
            start_position = calculate_end_position(start_position, slice);
            let old_end_position =
                calculate_end_position(start_position, &self.content[start_byte..old_end_byte]);
            let new_end_position = calculate_end_position(start_position, &replacement);
            new.push_str(slice);
            new.push_str(&replacement);
            last_match = old_end_byte;

            edits.push(tree_sitter::InputEdit {
                start_byte,
                old_end_byte,
                new_end_byte,
                start_position,
                old_end_position,
                new_end_position,
            });

            start_position = old_end_position;
        }

        new.push_str(&self.content[last_match..]);
        self.content = new;

        for edit in edits {
            self.tree.edit(&edit);
        }
        self.tree = self.parser.parse(&self.content, Some(&self.tree)).unwrap();
    }

    /// This function makes sure we have the correct vertical spacing between important definitions:
    /// Two blank lines between function definitions, inner classes, etc. Taking any
    /// comments or docstrings into account.
    ///
    /// This uses tree-sitter to find the relevant nodes and their positions.
    fn handle_two_blank_line(&mut self) -> &mut Self {
        let root = self.tree.root_node();
        let queries = [
            // We need two queries to catch all cases because variables can be placed above or below functions
            // First query: variable, function, class, signal, const, enum followed by function, constructor, class, or variable
            //
            // NOTE: Nathan (GDQuest): This adds maybe 20-25% runtime to the program.
            // I tried 2 other implementations by having a single query that'd find only functions, classes, and constructors and add 2 new lines between them.
            // But the costly part is in accounting for comments and annotations between them. This solution ends up being slightly faster and simpler.
            // Still, this is probably something that can be made faster in the future.
            "(([(variable_statement) (function_definition) (class_definition) (signal_statement) (const_statement) (enum_definition) (constructor_definition)]) @first \
            . (([(comment) (annotation)])* @comment . ([(function_definition) (constructor_definition) (class_definition)]) @second))",
            // Second query: constructor or function followed by variable, signal, const, or enum
            "(([(constructor_definition) (function_definition) (class_definition)]) @first \
            . ([(variable_statement) (signal_statement) (const_statement) (enum_definition)]) @second)",
        ];

        let process_query =
            |query_str: &str, new_lines_at: &mut Vec<(usize, tree_sitter::Point)>| {
                let query = match Query::new(
                    &tree_sitter::Language::new(tree_sitter_gdscript::LANGUAGE),
                    query_str,
                ) {
                    Ok(q) => q,
                    Err(err) => {
                        panic!("Failed to create query: {}", err);
                    }
                };

                let mut cursor = QueryCursor::new();
                let mut matches = cursor.matches(&query, root, self.content.as_bytes());
                while let Some(m) = matches.next() {
                    let first_node = m.captures[0].node;
                    let last_node = m.captures.last().unwrap().node;

                    let mut insert_before = last_node;

                    let capture_has_comments = m.captures.len() >= 3;

                    if capture_has_comments {
                        let last_comment_node = m.captures[m.captures.len() - 2].node;

                        let last_comment_is_inline_comment = last_comment_node.start_position().row
                            == first_node.start_position().row;
                        let last_comment_is_doc_comment = !last_comment_is_inline_comment
                            && last_comment_node.start_position().row
                                == last_node.start_position().row - 1;

                        // if last comment node is a doc comment find first doc comment node and insert new lines before that
                        if last_comment_is_doc_comment {
                            let mut comment_node_index = m.captures.len() - 2;

                            let first_comment_node = m.captures[1].node;
                            let first_comment_is_inline_comment =
                                first_comment_node.start_position().row
                                    == first_node.start_position().row;
                            // ignore n first nodes when searching for the first docstring comment node
                            // in case if the first comment is an inline comment we ignore
                            // two nodes: first statement node and inline comment node
                            // otherwise we ignore only the first statement node
                            let mut amount_of_nodes_to_ignore = 1;
                            if first_comment_is_inline_comment {
                                amount_of_nodes_to_ignore += 1;
                            }

                            // find first documentation comment node
                            while comment_node_index > amount_of_nodes_to_ignore
                                && m.captures[comment_node_index - 1].node.start_position().row
                                    == m.captures[comment_node_index].node.start_position().row - 1
                            {
                                comment_node_index -= 1;
                            }
                            insert_before = m.captures[comment_node_index].node;
                        }
                    }

                    let mut byte_idx = insert_before.start_byte();
                    let mut position = insert_before.start_position();
                    position.column = 0;
                    while byte_idx > 0 && self.content.as_bytes()[byte_idx] != b'\n' {
                        byte_idx -= 1;
                    }
                    new_lines_at.push((byte_idx, position));
                }
            };

        // First we need to find all the places where we should add blank lines.
        // We can't modify the content string while tree-sitter is borrowing it, so we
        // collect all the positions first, then make changes afterward.
        let mut new_lines_at = Vec::new();

        for query_str in &queries {
            process_query(query_str, &mut new_lines_at);
        }

        // We sort the positions in reverse order so that when we insert new lines,
        // we don't mess up the positions of the other insertions we need to make.
        new_lines_at.sort_by(|a, b| b.cmp(a));

        for (byte_idx, position) in new_lines_at {
            let mut new_end_position = position;
            let mut new_end_byte_idx = byte_idx;
            // Only add a second blank line if there isn't already one
            if !(self.content.as_bytes()[byte_idx] == b'\n'
                && self.content.as_bytes()[byte_idx - 1] == b'\n')
            {
                new_end_position.row += 1;
                new_end_byte_idx += 1;
                self.content.insert(byte_idx, '\n');
            }
            // Add the first blank line
            new_end_position.row += 1;
            new_end_byte_idx += 1;
            self.content.insert(byte_idx, '\n');

            // Update the tree sitter parse tree to reflect our changes so that any
            // future processing will work with the correct positions
            self.tree.edit(&tree_sitter::InputEdit {
                start_byte: byte_idx,
                old_end_byte: byte_idx,
                new_end_byte: new_end_byte_idx,
                start_position: position,
                old_end_position: position,
                new_end_position,
            });
        }
        self
    }
}

/// A syntax tree of the source code.
struct GdTree {
    nodes: Vec<GdTreeNode>,
}

impl GdTree {
    /// Constructs a new `GdTree` from `TSTree`.
    fn from_ts_tree(tree: &Tree, source: &[u8]) -> Self {
        let mut cursor = tree.walk();
        let mut nodes = Vec::new();

        let ts_root = cursor.node();

        let root = GdTreeNode {
            parent_id: None,
            grammar_id: ts_root.grammar_id(),
            grammar_name: ts_root.grammar_name(),
            text: None,
            children: Vec::new(),
        };
        nodes.push(root);

        let mut queue = VecDeque::new();
        queue.push_back((ts_root, 0));

        while let Some((parent_ts_node, parent_node_id)) = queue.pop_front() {
            let ts_children = parent_ts_node.children(&mut cursor);
            for ts_child in ts_children {
                // Skip anonymous nodes
                if !ts_child.is_named() {
                    continue;
                }

                // Get node's text in the source code (e.g. variable's name)
                // None if this node is not a leaf node
                let text = if ts_child.child(0).is_none() {
                    let range = ts_child.range();
                    Some(
                        str::from_utf8(&source[range.start_byte..range.end_byte])
                            .unwrap()
                            .to_string(),
                    )
                } else {
                    None
                };

                let child_id = nodes.len();
                let child = GdTreeNode {
                    parent_id: Some(parent_node_id),
                    grammar_id: ts_child.grammar_id(),
                    grammar_name: ts_child.grammar_name(),
                    text,
                    children: Vec::new(),
                };
                nodes.push(child);

                let parent_node = &mut nodes[parent_node_id];
                parent_node.children.push(child_id);

                queue.push_back((ts_child, child_id));
            }
        }

        GdTree { nodes }
    }

    fn postprocess(&mut self) {
        // During formatting we make changes that modify the syntax tree, some of these changes are expected,
        // so we have to adjust the syntax tree in order for safe mode to work properly.
        self.move_extends_statement();
        self.move_annotations();
    }

    /// Moves `extends_statement` to be a direct sibling of `class_name_statement` instead of its child.
    fn move_extends_statement(&mut self) {
        // Since class_name is always at the top level of the tree, we need to only iterate over root's children
        for child_index in (0..self.nodes[0].children.len()).rev() {
            let child_id = self.nodes[0].children[child_index];
            let child = &self.nodes[child_id];

            // We first search for a class_name_statement node
            if child.grammar_name != "class_name_statement" {
                continue;
            }

            // If this class extends from anything, extends_statement will be the second child,
            // because the first child will be the name of the class
            if child.children.len() < 2 {
                continue;
            }

            let second_child_id = child.children[1];
            let second_child = &self.nodes[second_child_id];

            if second_child.grammar_name != "extends_statement" {
                continue;
            }

            // When we found it, we move it to be a direct sibling of class_name_statement node
            let class_name_node = &mut self.nodes[child_id];
            let extends_node_id = class_name_node.children.remove(1);

            let root = &mut self.nodes[0];
            root.children.insert(child_index + 1, extends_node_id);

            let extends_node = &mut self.nodes[extends_node_id];
            extends_node.parent_id = Some(0);
        }
    }

    fn move_annotations(&mut self) {
        let language: &tree_sitter::Language = &tree_sitter_gdscript::LANGUAGE.into();
        let annotations_grammar_id = language.id_for_node_kind("annotations", true);

        let mut stack = Vec::new();
        stack.push(0);

        while let Some(parent_id) = stack.pop() {
            // We need to modify the index when we delete nodes
            let mut index = self.nodes[parent_id].children.len();
            while index > 0 {
                index -= 1;
                let child_id = self.nodes[parent_id].children[index];
                let child_grammar_name = self.nodes[child_id].grammar_name;

                // We do the same in inner classes
                if child_grammar_name == "class_definition" {
                    stack.push(child_id);
                    continue;
                }

                if child_grammar_name == "variable_statement" {
                    // We move @onready and @export annotations on the same line as the variable after formatting,
                    // that means we need to move these annotations to be children of the variable_statement node
                    // We move from the current index back to 0, searching for any annotations
                    let annotations_to_move = (0..index)
                        .rev()
                        .map_while(|i| {
                            let child_id = self.nodes[parent_id].children[i];
                            let child = &self.nodes[child_id];
                            if child.grammar_name != "annotation" {
                                return None;
                            }
                            let annotation_name =
                                self.nodes[child.children[0]].text.as_deref().unwrap();
                            if annotation_name != "onready" && annotation_name != "export" {
                                return None;
                            }
                            let parent = &mut self.nodes[parent_id];
                            // When we found one, we remove it from the parent and collect them in a vector
                            let annotation_id = parent.children.remove(i);
                            index -= 1;
                            Some(annotation_id)
                        })
                        .collect::<Vec<_>>();

                    if annotations_to_move.is_empty() {
                        continue;
                    }

                    let mut annotations_node_exists = false;

                    let variable_node = &self.nodes[child_id];
                    let variable_first_child_id = variable_node.children[0];
                    let variable_first_child = &mut self.nodes[variable_first_child_id];

                    let (annotations_node, annotations_node_id) =
                        // If the first child is (annotations) node, then we add annotations to it
                        if variable_first_child.grammar_name == "annotations" {
                            annotations_node_exists = true;
                            (variable_first_child, variable_first_child_id)
                        // If variable doesn't already have (annotations) node, we create a new one
                        } else {
                            let annotations = GdTreeNode {
                                parent_id: Some(child_id),
                                grammar_id: annotations_grammar_id,
                                grammar_name: "annotations",
                                text: None,
                                children: Vec::new(),
                            };
                            let annotations_id = self.nodes.len();
                            self.nodes.push(annotations);
                            (&mut self.nodes[annotations_id], annotations_id)
                        };

                    for annotation_id in annotations_to_move {
                        annotations_node.children.insert(0, annotation_id);
                    }

                    if !annotations_node_exists {
                        let variable_node = &mut self.nodes[child_id];
                        variable_node.children.insert(0, annotations_node_id);
                    }
                }
            }
        }
    }
}

impl PartialEq for GdTree {
    fn eq(&self, other: &Self) -> bool {
        let mut left_stack = Vec::new();
        let mut right_stack = Vec::new();

        // Starting from root (0)
        left_stack.push(0);
        right_stack.push(0);

        while let (Some(left_current_node_id), Some(right_current_node_id)) =
            (left_stack.pop(), right_stack.pop())
        {
            let left_current_node = &self.nodes[left_current_node_id];
            let right_current_node = &other.nodes[right_current_node_id];
            if left_current_node.children.len() != right_current_node.children.len() {
                // A different number of children means the syntax trees are different, so the code
                // structure has changed.
                // NOTE: There's a valid case of change: an annotation above a variable may be wrapped
                // on the same line as the variable, which turns the annotation into a child of the variable.
                // We could ignore this specific case, but for now, we consider any change in structure
                // as a potential issue.
                return false;
            }

            for (left_node_id, right_node_id) in left_current_node
                .children
                .iter()
                .zip(right_current_node.children.iter())
            {
                let left_node = &self.nodes[*left_node_id];
                let right_node = &other.nodes[*right_node_id];
                if left_node.grammar_id != right_node.grammar_id {
                    return false;
                }
                left_stack.push(*left_node_id);
                right_stack.push(*right_node_id);
            }
        }
        true
    }
}

struct GdTreeNode {
    parent_id: Option<usize>,
    grammar_id: u16,
    grammar_name: &'static str,
    text: Option<String>,
    children: Vec<usize>,
}

/// Calculates end position of the `slice` counting from `start`
fn calculate_end_position(mut start: Point, slice: &str) -> Point {
    for b in slice.as_bytes() {
        if *b == b'\n' {
            start.row += 1;
            start.column = 0;
        } else {
            start.column += 1;
        }
    }
    start
}
