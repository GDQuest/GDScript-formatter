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
use std::io::BufWriter;

use regex::{Regex, RegexBuilder};
use topiary_core::{Language, Operation, TopiaryQuery, formatter_tree};
use tree_sitter::{Parser, Point, Query, QueryCursor, StreamingIterator, Tree};

use crate::FormatterConfig;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum SpacingType {
    Single,
    Double,
}

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
    input_tree: Tree,
    tree: Tree,
}

impl Formatter {
    #[inline(always)]
    fn new(content: String, config: FormatterConfig) -> Self {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_gdscript::LANGUAGE.into())
            .unwrap();
        let input_tree = parser.parse(&content, None).unwrap();

        Self {
            content,
            config,
            tree: input_tree.clone(),
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
        self.remove_newlines_after_extends_statement()
    }

    /// This function runs over the content after going through topiary. We use it
    /// to clean up/balance out the output.
    #[inline(always)]
    fn postprocess(&mut self) -> &mut Self {
        self.fix_dangling_semicolons()
            .fix_dangling_commas()
            .remove_trailing_commas_from_preload()
            .postprocess_tree_sitter()
    }

    /// Finishes formatting and returns the resulting file content.
    #[inline(always)]
    fn finish(mut self) -> Result<String, Box<dyn std::error::Error>> {
        if self.config.safe {
            self.tree = self.parser.parse(&self.content, Some(&self.tree)).unwrap();

            if !compare_trees(self.input_tree, self.tree) {
                return Err("Trees are different".into());
            }
        }

        Ok(self.content)
    }

    /// This function removes additional new line characters after `extends_statement`.
    #[inline(always)]
    fn remove_newlines_after_extends_statement(&mut self) -> &mut Self {
        // This regex matches substrings which:
        // - must NOT contain "#" or "\n" characters between new line and "extends" keyword
        // - must end with at least one new line character
        // - must contain `extends_name` character sequence that satisfies one of the following conditions:
        //   - consists out of alphanumeric characters
        //   - consists out of any characters (except new lines) between double quotes
        let re = RegexBuilder::new(
            r#"(?P<extends_line>^[^#\n]*extends )(?P<extends_name>([a-zA-Z0-9]+|".*?"))\n(\n*)"#,
        )
        .multi_line(true)
        .build()
        .expect("regex should compile");

        self.regex_replace_all_outside_strings(re, "$extends_line$extends_name\n");
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
    fn regex_replace_all_outside_strings(&mut self, re: Regex, rep: &str) {
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
            capture.expand(rep, &mut replacement);

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
            |query_str: &str, new_lines_at: &mut Vec<(usize, tree_sitter::Point, SpacingType)>| {
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
                    if m.captures.len() == 3 {
                        let comment_node = m.captures[1].node;
                        let second_node = m.captures[2].node;
                        
                        // Check if the comment immediately follows the first node on the next line
                        // (indicating it's a trailing comment belonging to the first node)
                        let first_end_row = first_node.end_position().row;
                        let comment_start_row = comment_node.start_position().row;
                        let is_next_line_trailing_comment = comment_node.kind() == "comment" && 
                                                           comment_start_row == first_end_row + 1;
                        
                        // If the @comment is on the same line as the first node,
                        // we'll add a blank line before the @second node
                        if comment_node.start_position().row == first_node.start_position().row {
                            // Find where to insert the new line (before any indentation)
                            let mut byte_idx = second_node.start_byte();
                            let mut position = second_node.start_position();
                            position.column = 0;
                            while self.content.as_bytes()[byte_idx] != b'\n' {
                                byte_idx -= 1;
                            }
                            new_lines_at.push((byte_idx, position, SpacingType::Double));
                        } else if is_next_line_trailing_comment {
                            // If the comment is on the line immediately after the first node,
                            // treat it as a trailing comment and only add one blank line
                            new_lines_at.push((comment_node.end_byte(), comment_node.end_position(), SpacingType::Single));
                        } else {
                            // Otherwise, add a blank line after the first node
                            new_lines_at.push((first_node.end_byte(), first_node.end_position(), SpacingType::Double));
                        }
                    } else {
                        // If there's no comment between the nodes, add a blank line after the first node
                        new_lines_at.push((first_node.end_byte(), first_node.end_position(), SpacingType::Double));
                    }
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

        for (byte_idx, position, spacing_type) in new_lines_at {
            let mut new_end_position = position;
            let mut new_end_byte_idx = byte_idx;
            
            match spacing_type {
                SpacingType::Double => {
                    // Only add a second blank line if there isn't already one
                    if self.content.as_bytes()[byte_idx + 1] != b'\n' {
                        new_end_position.row += 1;
                        new_end_byte_idx += 1;
                        self.content.insert(byte_idx, '\n');
                    }
                }
                SpacingType::Single => {
                    // For single spacing, we only add one blank line
                }
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

/// Returns true if both trees have the same structure.
fn compare_trees(left_tree: Tree, right_tree: Tree) -> bool {
    let mut left_cursor = left_tree.walk();
    let mut right_cursor = right_tree.walk();

    let mut left_stack = Vec::new();
    let mut right_stack = Vec::new();
    left_stack.push(left_cursor.node());
    right_stack.push(right_cursor.node());

    while let (Some(left_current_node), Some(right_current_node)) =
        (left_stack.pop(), right_stack.pop())
    {
        if left_current_node.child_count() != right_current_node.child_count() {
            // A different number of children means the syntax trees are different, so the code
            // structure has changed.
            // NOTE: There's a valid case of change: an annotation above a variable may be wrapped
            // on the same line as the variable, which turns the annotation into a child of the variable.
            // We could ignore this specific case, but for now, we consider any change in structure
            // as a potential issue.
            return false;
        }

        let left_children = left_current_node.children(&mut left_cursor);
        let right_children = left_current_node.children(&mut right_cursor);
        for (left_node, right_node) in left_children.zip(right_children) {
            if left_node.grammar_id() != right_node.grammar_id() {
                return false;
            }
            left_stack.push(left_node);
            right_stack.push(right_node);
        }
    }
    true
}
