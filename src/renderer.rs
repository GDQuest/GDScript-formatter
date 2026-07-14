//! This module takes a `RenderElement` struct (intermediate representation) and
//! renders the formatted GDScript code into a string.
//!
//! We use Philip Wadler's pretty printing algorithm: the renderer walks a flat
//! list of `RenderElement` nodes and tries to fit the children of each `Group` node
//! on one line. If a group's contents is longer than the desired maximum line
//! width, the renderer breaks it across multiple lines instead.
//!
//! Tabs count as `indent_size` columns for width measurement (1 tab = 4 spaces
//! by default). Blank lines are always merged or limited to a maximum (2 blank
//! lines by default) and the output always ends with a trailing newline.
//!
//! Wadler's paper for reference:
//! https://homepages.inf.ed.ac.uk/wadler/papers/prettier/prettier.pdf

/// A byte range in the original source text. Both fields are byte offsets and
/// end_byte is exclusive.
pub struct RangeSourceBytes {
    pub start_byte: usize,
    pub end_byte: usize,
}

/// A range of indices to use on a RenderElement list.
/// `end` is exclusive.
pub struct RangeRenderElement {
    pub start: usize,
    pub end: usize,
}

pub enum RenderElement {
    Text {
        range: RangeSourceBytes,
    },
    TextStatic(&'static str),
    /// Text produced or edited by the formatter rather than copied from the
    /// source. Used if the user set the option to change quote style, to edit
    /// the quotes of input strings.
    TextProducedByFormatter(String),
    /// Represents a single space character.
    Space,
    /// Represents an optional line return that may be output at render time (if
    /// needed, e.g. if a line is too long, a line return can be applied at this
    /// token).
    SoftLine,
    /// Represents a mandatory line return that should always be applied (a single \n).
    HardLine,
    /// Represents a blank (empty) line that should be output (a double line return: \n\n).
    BlankLine,
    /// This represents an optional space: when a group fits on a line, this
    /// inserts a space. However if the line is too long and line returns are
    /// needed, this adds nothing to the output.
    ///
    /// This is used for cases like single-line dictionaries and enums as they
    /// need a space around braces (e.g. enum States { IDLE, RUNNING })
    SpaceSingleLineOnly,
    /// Represents an indentation level that should be applied to a range of render elements.
    Indent {
        level: u16,
        child: RangeRenderElement,
    },
    /// Groups a set of tokens together for the line wrapping algorithm.
    Group {
        children: RangeRenderElement,
    },
    /// Describes two possible sequences of tokens to render: one if the
    /// currently rendered line fits on a single line, and one if it exceeds max
    /// line length and should be broken into multiple lines.
    Branch {
        if_single_line: Option<RangeRenderElement>,
        if_multiline: Option<RangeRenderElement>,
    },
    /// Forces the closest parent (enclosing) group to break into multiline mode
    /// (turn softlines into line breaks).
    ///
    /// When the renderer is checking if a group fits on one line, when it sees
    /// this element, it stops and returns "does not fit". This only affects the
    /// direct parent group, not any nested groups.
    ///
    /// Used by containers with multiline content, multiline ternary
    /// expressions, and lambdas with multiline bodies.
    ForceBreakingParent,
    /// Represents a source range that should be output as-is, without any
    /// formatting applied. Also used for `# fmt: off` disabled regions: the
    /// range starts at the off marker's `#` (not the start of its line), so
    /// the renderer's normal indent handling applies to the marker line and
    /// everything after it is emitted verbatim.
    UnformattedSource {
        range: RangeSourceBytes,
    },
}

#[derive(Clone)]
pub struct PrinterConfiguration {
    /// The maximum line length for the formatter (default: 100).
    pub max_line_length: usize,
    /// Use spaces for indentation (default: false, GDScript uses tabs by
    /// default).
    pub use_spaces: bool,
    /// The size of each indent level (default: 4).
    pub indent_size: usize,
    /// The maximum number of blank lines to insert (default: 2).
    pub maximum_blank_lines: u16,
    /// Extra indent level applied to continuation lines (default: 2).
    pub continuation_indent_level: u16,
    /// Ensure output ends with a single newline (on by default).
    pub insert_final_newline: bool,
    /// Remove trailing whitespace from lines (on by default).
    pub trim_trailing_whitespace: bool,
    /// Preserve and normalize block-level indentation on blank lines (off by
    /// default). Enable this when using the built-in script editor in Godot, as
    /// it does not have visual indent guides. This will keep tabs and make it
    /// easier to see the indentation structure of the code.
    pub indent_blank_lines: bool,
}

impl Default for PrinterConfiguration {
    fn default() -> Self {
        Self {
            max_line_length: 100,
            use_spaces: false,
            indent_size: 4,
            maximum_blank_lines: 2,
            continuation_indent_level: 2,
            insert_final_newline: true,
            trim_trailing_whitespace: true,
            indent_blank_lines: false,
        }
    }
}

pub fn render(
    render_elements: &[RenderElement],
    source: &str,
    config: &PrinterConfiguration,
    output: &mut String,
) {
    output.clear();
    output.reserve(source.len());
    let spaces;
    let indent_unit: &str = if config.use_spaces {
        spaces = " ".repeat(config.indent_size);
        &spaces
    } else {
        "\t"
    };
    let mut printer = Printer {
        render_elements,
        source,
        config,
        indent_unit,
        output: std::mem::take(output),
        column: 0,
        pending_newlines: 0u16,
        indent_level: 0,
    };
    printer.render_range(0, render_elements.len(), Mode::Flat);
    *output = printer.finish();
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Flat,
    Break,
}

struct Printer<'a> {
    render_elements: &'a [RenderElement],
    source: &'a str,
    config: &'a PrinterConfiguration,
    indent_unit: &'a str,
    output: String,
    column: usize,
    pending_newlines: u16,
    indent_level: u16,
}

impl<'a> Printer<'a> {
    fn render_range(&mut self, start: usize, end: usize, mode: Mode) {
        let mut index = start;
        while index < end {
            match &self.render_elements[index] {
                RenderElement::Text { range } => {
                    let text = slice(self.source, range);
                    self.emit_text(text);
                    index += 1;
                }
                RenderElement::TextStatic(text) => {
                    self.emit_text(text);
                    index += 1;
                }
                RenderElement::TextProducedByFormatter(text) => {
                    self.emit_text(text);
                    index += 1;
                }
                RenderElement::Space => {
                    self.emit_text(" ");
                    index += 1;
                }
                RenderElement::SpaceSingleLineOnly => {
                    if mode == Mode::Flat {
                        self.emit_text(" ");
                    }
                    index += 1;
                }
                RenderElement::SoftLine => {
                    if mode == Mode::Break {
                        self.request_newline(1);
                    }
                    index += 1;
                }
                RenderElement::HardLine => {
                    self.request_newline(1);
                    index += 1;
                }
                RenderElement::BlankLine => {
                    let max_lines = self.config.maximum_blank_lines + 1;
                    let mut want = self.pending_newlines + 1;
                    if want < 2 {
                        want = 2;
                    }
                    if want > max_lines {
                        want = max_lines;
                    }
                    self.request_newline(want);
                    index += 1;
                }
                RenderElement::Indent { level, child } => {
                    self.indent_level += *level;
                    self.render_range(child.start, child.end, mode);
                    self.indent_level -= *level;
                    index = child.end;
                }
                RenderElement::Group { children } => {
                    let child_mode = self.decide_group_render_mode(children.start, children.end);
                    self.render_range(children.start, children.end, child_mode);
                    index = children.end;
                }
                RenderElement::Branch {
                    if_single_line: flat,
                    if_multiline: break_,
                } => {
                    let selected = if mode == Mode::Break { break_ } else { flat };
                    if let Some(range) = selected {
                        self.render_range(range.start, range.end, mode);
                    }
                    index = skip_past_branch(index, flat, break_);
                }
                RenderElement::ForceBreakingParent => {
                    index += 1;
                }
                RenderElement::UnformattedSource { range } => {
                    let text = slice(self.source, range);
                    // Insert raw text to the output without any formatting
                    // while tracking the current column and pending newlines to
                    // keep track of indentation.
                    self.flush_newlines();
                    for c in text.chars() {
                        if c == '\n' {
                            self.output.push('\n');
                            self.column = 0;
                            self.pending_newlines = 0;
                        } else {
                            self.output.push(c);
                            if c == '\t' {
                                self.column += self.config.indent_size;
                            } else {
                                self.column += 1;
                            }
                        }
                    }
                    index += 1;
                }
            }
        }
    }

    fn request_newline(&mut self, want: u16) {
        if want > self.pending_newlines {
            self.pending_newlines = want;
        }
    }

    fn decide_group_render_mode(&self, start: usize, end: usize) -> Mode {
        let column = if self.pending_newlines > 0 {
            self.indent_level as usize * self.config.indent_size
        } else {
            self.column
        };
        let mut current_column = column;
        if self.does_group_fit_on_one_line(start, end, &mut current_column, true) {
            Mode::Flat
        } else {
            Mode::Break
        }
    }

    /// Checks whether a range of node's contents fit on a single line (recursively).
    /// `check_force_break` controls whether a `ForceBreakingParent` element
    /// aborts the check: when true (the direct parent group), encountering one
    /// returns false; when false (a nested group being measured), it is ignored.
    fn does_group_fit_on_one_line(
        &self,
        start: usize,
        end: usize,
        column: &mut usize,
        check_force_break: bool,
    ) -> bool {
        let mut index = start;
        while index < end {
            match &self.render_elements[index] {
                RenderElement::Text { range } | RenderElement::UnformattedSource { range } => {
                    if !self.measure_text(slice(self.source, range), column) {
                        return false;
                    }
                    index += 1;
                }
                RenderElement::TextStatic(text) => {
                    if !self.measure_text(text, column) {
                        return false;
                    }
                    index += 1;
                }
                RenderElement::TextProducedByFormatter(text) => {
                    if !self.measure_text(text, column) {
                        return false;
                    }
                    index += 1;
                }
                RenderElement::Space | RenderElement::SpaceSingleLineOnly => {
                    *column += 1;
                    if *column > self.config.max_line_length {
                        return false;
                    }
                    index += 1;
                }
                RenderElement::SoftLine => index += 1,
                RenderElement::HardLine | RenderElement::BlankLine => return false,
                RenderElement::Indent { child, .. } => {
                    if !self.does_group_fit_on_one_line(
                        child.start,
                        child.end,
                        column,
                        check_force_break,
                    ) {
                        return false;
                    }
                    index = child.end;
                }
                RenderElement::Group { children } => {
                    if !self.does_group_fit_on_one_line(children.start, children.end, column, false)
                    {
                        return false;
                    }
                    index = children.end;
                }
                RenderElement::Branch {
                    if_single_line: flat,
                    if_multiline: break_,
                } => {
                    if let Some(range) = flat
                        && !self.does_group_fit_on_one_line(
                            range.start,
                            range.end,
                            column,
                            check_force_break,
                        )
                    {
                        return false;
                    }
                    index = skip_past_branch(index, flat, break_);
                }
                RenderElement::ForceBreakingParent => {
                    if check_force_break {
                        return false;
                    }
                    index += 1;
                }
            }
        }
        true
    }

    fn measure_text(&self, text: &str, column: &mut usize) -> bool {
        for c in text.chars() {
            if c == '\n' {
                return false;
            }
            if c == '\t' {
                *column += self.config.indent_size;
            } else {
                *column += 1;
            }
            if *column > self.config.max_line_length {
                return false;
            }
        }
        true
    }

    fn emit_text(&mut self, text: &str) {
        for c in text.chars() {
            if c == '\n' {
                self.pending_newlines += 1;
            } else {
                self.flush_newlines();
                self.output.push(c);
                if c == '\t' {
                    self.column += self.config.indent_size;
                } else {
                    self.column += 1;
                }
            }
        }
    }

    fn flush_newlines(&mut self) {
        if self.pending_newlines == 0 {
            return;
        }
        if self.output.is_empty() && self.pending_newlines > 1 {
            self.pending_newlines = 1;
        }
        // Trim trailing whitespace from the current line before inserting
        // newline characters. I.e. turns "text \n" into "text\n".
        if self.config.trim_trailing_whitespace {
            let trimmed_len = self.output.trim_end_matches([' ', '\t']).len();
            self.output.truncate(trimmed_len);
        }
        // When indent_blank_lines is true and there are blank lines (i.e., pending
        // newlines > 1), try to preserve leading indentation on the blank lines.
        //
        // This is for Godot users as Godot's script editor shows tab characters
        // and not indent level lines at the time of writing.
        //
        // See https://github.com/GDQuest/GDScript-formatter/issues/151 for
        // discussion and examples.
        if self.config.indent_blank_lines && self.pending_newlines > 1 {
            self.output.push('\n');
            let mut processed_blank_line_count = 1;
            while processed_blank_line_count < self.pending_newlines {
                for _ in 0..self.indent_level {
                    self.output.push_str(self.indent_unit);
                }
                self.output.push('\n');
                processed_blank_line_count += 1;
            }
        } else {
            for _ in 0..self.pending_newlines {
                self.output.push('\n');
            }
        }
        for _ in 0..self.indent_level {
            self.output.push_str(self.indent_unit);
        }
        self.column = self.indent_level as usize * self.config.indent_size;
        self.pending_newlines = 0;
    }

    fn finish(mut self) -> String {
        if !self.config.insert_final_newline {
            return self.output;
        }
        if self.output.is_empty() {
            return self.output;
        }
        if !self.output.ends_with('\n') {
            self.output.push('\n');
        }
        self.output
    }
}

fn skip_past_branch(
    index: usize,
    flat: &Option<RangeRenderElement>,
    break_: &Option<RangeRenderElement>,
) -> usize {
    let mut skip_to = index + 1;
    if let Some(range) = flat
        && range.end > skip_to
    {
        skip_to = range.end;
    }
    if let Some(range) = break_
        && range.end > skip_to
    {
        skip_to = range.end;
    }
    skip_to
}

fn slice<'a>(source: &'a str, range: &RangeSourceBytes) -> &'a str {
    &source[range.start_byte..range.end_byte]
}
