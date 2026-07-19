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

/// Controls how an enclosing group includes this child group's contents in its
/// flat-layout fit check. This affects whether the enclosing group uses flat
/// or broken layout; the child group always chooses its own layout separately.
pub enum GroupParentFit {
    /// Include this group's complete flat layout in the parent group's flat
    /// layout measurement.
    Full,
    /// Stop including this group's contents after its first potential line
    /// break: a soft line, hard line, or blank line. This group still
    /// independently chooses and renders its complete layout.
    ///
    /// This keeps a property or method chain together when a method argument
    /// must be multiline, like a lambda:
    ///
    /// ```gdscript
    /// f.args.map(
    ///     func(arg):
    ///         arg.erase("name")
    /// )
    /// ```
    ///
    /// The parent chain stays flat through `f.args.map(`. The `map()` argument
    /// group then breaks and formats the lambda normally.
    UntilFirstLineBreak,
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
    /// A space between segments in a `BalancedGroup`. Selected balanced lines
    /// become line returns when the group breaks.
    BalancedLine,
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
        parent_fit: GroupParentFit,
    },
    /// Groups segments separated by `BalancedLine` and distributes them across
    /// lines when they don't fit a flat layout. The renderer tries to
    /// distribute the contents of a balanced group across multiple lines so
    /// that lines are roughly the same length. Currently used mainly for
    /// long chains of binary operator expressions.
    BalancedGroup {
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
        balanced_break_plans: Vec::new(),
    };
    printer.render_range(0, render_elements.len(), Mode::Flat);
    *output = printer.add_to_output_finish();
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Flat,
    Break,
}

#[derive(Clone, Copy, PartialEq)]
enum ForceBreakMode {
    Ignore,
    DirectChildren,
    AnyDepth,
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
    balanced_break_plans: Vec<Vec<usize>>,
}

impl<'a> Printer<'a> {
    fn render_range(&mut self, start: usize, end: usize, mode: Mode) {
        let mut index = start;
        while index < end {
            match &self.render_elements[index] {
                RenderElement::Text { range } => {
                    let text = slice(self.source, range);
                    self.add_to_output(text);
                    index += 1;
                }
                RenderElement::TextStatic(text) => {
                    self.add_to_output(text);
                    index += 1;
                }
                RenderElement::TextProducedByFormatter(text) => {
                    self.add_to_output(text);
                    index += 1;
                }
                RenderElement::Space => {
                    self.add_to_output(" ");
                    index += 1;
                }
                RenderElement::SpaceSingleLineOnly => {
                    if mode == Mode::Flat {
                        self.add_to_output(" ");
                    }
                    index += 1;
                }
                RenderElement::SoftLine => {
                    if mode == Mode::Break {
                        self.request_newline(1);
                    }
                    index += 1;
                }
                RenderElement::BalancedLine => {
                    if mode == Mode::Flat {
                        self.add_to_output(" ");
                    } else {
                        let mut should_break = false;
                        if let Some(breaks) = self.balanced_break_plans.last_mut()
                            && breaks.last() == Some(&index)
                        {
                            breaks.pop();
                            should_break = true;
                        }
                        if should_break {
                            self.request_newline(1);
                        } else if self.pending_newlines == 0 {
                            self.add_to_output(" ");
                        }
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
                RenderElement::Group { children, .. } => {
                    let child_mode = self.decide_group_render_mode(
                        children.start,
                        children.end,
                        ForceBreakMode::DirectChildren,
                    );
                    self.render_range(children.start, children.end, child_mode);
                    index = children.end;
                }
                RenderElement::BalancedGroup { children } => {
                    let child_mode = self.decide_group_render_mode(
                        children.start,
                        children.end,
                        ForceBreakMode::AnyDepth,
                    );
                    if child_mode == Mode::Flat {
                        self.render_range(children.start, children.end, Mode::Flat);
                    } else {
                        let mut breaks = self.plan_balanced_breaks(children.start, children.end);
                        breaks.reverse();
                        self.balanced_break_plans.push(breaks);
                        self.render_range(children.start, children.end, Mode::Break);
                        self.balanced_break_plans.pop();
                    }
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
                    self.add_to_output_process_newlines();
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

    fn decide_group_render_mode(
        &self,
        start: usize,
        end: usize,
        force_break_mode: ForceBreakMode,
    ) -> Mode {
        let column = if self.pending_newlines > 0 {
            self.indent_level as usize * self.config.indent_size
        } else {
            self.column
        };
        let mut current_column = column;
        if self.does_group_fit_on_one_line(start, end, &mut current_column, force_break_mode) {
            Mode::Flat
        } else {
            Mode::Break
        }
    }

    /// Checks whether a range of node's contents fit on a single line (recursively).
    /// `force_break_mode` controls whether `ForceBreakingParent` aborts the
    /// check for direct children, at any depth, or not at all.
    fn does_group_fit_on_one_line(
        &self,
        start: usize,
        end: usize,
        column: &mut usize,
        force_break_mode: ForceBreakMode,
    ) -> bool {
        self.does_range_fit_flat_layout(start, end, column, force_break_mode)
            && *column <= self.config.max_line_length
    }

    /// Measures the width of a range of render elements on a single line and
    /// returns true if the content would fit the line. Returns false when the
    /// range contains content that cannot render on one physical line.
    fn does_range_fit_flat_layout(
        &self,
        start: usize,
        end: usize,
        column: &mut usize,
        force_break_mode: ForceBreakMode,
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
                RenderElement::Space
                | RenderElement::SpaceSingleLineOnly
                | RenderElement::BalancedLine => {
                    *column = column.saturating_add(1);
                    index += 1;
                }
                RenderElement::SoftLine => index += 1,
                RenderElement::HardLine | RenderElement::BlankLine => return false,
                RenderElement::Indent { child, .. } => {
                    if !self.does_range_fit_flat_layout(
                        child.start,
                        child.end,
                        column,
                        force_break_mode,
                    ) {
                        return false;
                    }
                    index = child.end;
                }
                RenderElement::Group {
                    children,
                    parent_fit: GroupParentFit::Full,
                }
                | RenderElement::BalancedGroup { children } => {
                    let nested_force_break_mode = if force_break_mode == ForceBreakMode::AnyDepth {
                        ForceBreakMode::AnyDepth
                    } else {
                        ForceBreakMode::Ignore
                    };
                    if !self.does_range_fit_flat_layout(
                        children.start,
                        children.end,
                        column,
                        nested_force_break_mode,
                    ) {
                        return false;
                    }
                    index = children.end;
                }
                RenderElement::Group {
                    children,
                    parent_fit: GroupParentFit::UntilFirstLineBreak,
                } => {
                    if !self.does_group_prefix_fit(children.start, children.end, column) {
                        return false;
                    }
                    index = children.end;
                }
                RenderElement::Branch {
                    if_single_line: flat,
                    if_multiline: break_,
                } => {
                    if let Some(range) = flat
                        && !self.does_range_fit_flat_layout(
                            range.start,
                            range.end,
                            column,
                            force_break_mode,
                        )
                    {
                        return false;
                    }
                    index = skip_past_branch(index, flat, break_);
                }
                RenderElement::ForceBreakingParent => {
                    if force_break_mode != ForceBreakMode::Ignore {
                        return false;
                    }
                    index += 1;
                }
            }
        }
        true
    }

    /// Measures a child group's contents until its first possible line break
    /// for its enclosing group's flat-layout fit check. The child group still
    /// chooses its complete layout independently.
    fn does_group_prefix_fit(&self, start: usize, end: usize, column: &mut usize) -> bool {
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
                RenderElement::Space
                | RenderElement::SpaceSingleLineOnly
                | RenderElement::BalancedLine => {
                    *column = column.saturating_add(1);
                    index += 1;
                }
                RenderElement::SoftLine | RenderElement::HardLine | RenderElement::BlankLine => {
                    return *column <= self.config.max_line_length;
                }
                RenderElement::Indent { child, .. }
                | RenderElement::Group {
                    children: child, ..
                }
                | RenderElement::BalancedGroup { children: child } => {
                    if !self.does_group_prefix_fit(child.start, child.end, column) {
                        return false;
                    }
                    index = child.end;
                }
                RenderElement::Branch {
                    if_single_line: flat,
                    if_multiline: break_,
                } => {
                    if let Some(range) = flat
                        && !self.does_group_prefix_fit(range.start, range.end, column)
                    {
                        return false;
                    }
                    index = skip_past_branch(index, flat, break_);
                }
                RenderElement::ForceBreakingParent => index += 1,
            }
            if *column > self.config.max_line_length {
                return false;
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
                *column = column.saturating_add(self.config.indent_size);
            } else {
                *column = column.saturating_add(1);
            }
        }
        true
    }

    fn plan_balanced_breaks(&self, start: usize, end: usize) -> Vec<usize> {
        let boundaries = {
            let mut found_boundaries = Vec::new();
            let mut index = start;
            while index < end {
                match &self.render_elements[index] {
                    RenderElement::BalancedLine => {
                        found_boundaries.push(index);
                        index += 1;
                    }
                    RenderElement::Indent { child, .. } => index = child.end,
                    RenderElement::Group { children, .. }
                    | RenderElement::BalancedGroup { children } => index = children.end,
                    RenderElement::Branch {
                        if_single_line,
                        if_multiline,
                    } => index = skip_past_branch(index, if_single_line, if_multiline),
                    _ => index += 1,
                }
            }
            found_boundaries
        };

        if boundaries.is_empty() {
            return Vec::new();
        }

        let segment_count = boundaries.len() + 1;
        let mut widths = Vec::with_capacity(segment_count);
        let mut multiline = Vec::with_capacity(segment_count);
        let mut segment_index = 0;
        while segment_index < segment_count {
            let segment_start = if segment_index == 0 {
                start
            } else {
                boundaries[segment_index - 1] + 1
            };
            let segment_end = if segment_index < boundaries.len() {
                boundaries[segment_index]
            } else {
                end
            };
            let mut width = 0;
            let fits_one_line = self.does_range_fit_flat_layout(
                segment_start,
                segment_end,
                &mut width,
                ForceBreakMode::AnyDepth,
            );
            widths.push(width);
            multiline.push(!fits_one_line);
            segment_index += 1;
        }

        let continuation_column = self.indent_level as usize * self.config.indent_size;
        let first_column = if self.pending_newlines > 0 {
            continuation_column
        } else {
            self.column
        };

        // First find the minimum line count by filling each line to its limit.
        let mut line_count = 1usize;
        let mut column = first_column;
        let mut line_has_segment = false;
        let mut previous_was_multiline = false;
        segment_index = 0;
        while segment_index < segment_count {
            if multiline[segment_index] {
                if line_has_segment {
                    line_count += 1;
                }
                line_has_segment = true;
                previous_was_multiline = true;
            } else {
                if previous_was_multiline {
                    line_count += 1;
                    column = continuation_column;
                    line_has_segment = false;
                    previous_was_multiline = false;
                }
                let separator_width = usize::from(line_has_segment);
                let next_column = column
                    .saturating_add(separator_width)
                    .saturating_add(widths[segment_index]);
                if line_has_segment && next_column > self.config.max_line_length {
                    line_count += 1;
                    column = continuation_column.saturating_add(widths[segment_index]);
                } else {
                    column = next_column;
                }
                line_has_segment = true;
            }
            segment_index += 1;
        }

        // Then try to place each break so that it gives an even share of the
        // remaining text. It's just a heuristic, we don't explore all possible
        // break points to find the best solution.
        let mut breaks = Vec::with_capacity(line_count.saturating_sub(1));
        let mut first_segment = 0;
        let mut remaining_lines = line_count;
        while remaining_lines > 1 && first_segment < segment_count {
            let line_start_column = if first_segment == 0 {
                first_column
            } else {
                continuation_column
            };
            let available_width = self
                .config
                .max_line_length
                .saturating_sub(line_start_column);
            let mut remaining_width = segment_count - first_segment - 1;
            let mut segment_index = first_segment;
            while segment_index < segment_count {
                remaining_width = remaining_width.saturating_add(widths[segment_index]);
                segment_index += 1;
            }
            let target_width = remaining_width
                .div_ceil(remaining_lines)
                .min(available_width);
            let last_allowed_segment = segment_count - remaining_lines;
            let mut next_segment = first_segment + 1;
            let mut line_width = widths[first_segment];

            if !multiline[first_segment] {
                while next_segment <= last_allowed_segment && !multiline[next_segment] {
                    let candidate_width = line_width
                        .saturating_add(1)
                        .saturating_add(widths[next_segment]);
                    let worsens_balance =
                        candidate_width.abs_diff(target_width) > line_width.abs_diff(target_width);
                    if candidate_width > available_width || worsens_balance {
                        break;
                    }
                    line_width = candidate_width;
                    next_segment += 1;
                }
            }

            breaks.push(boundaries[next_segment - 1]);
            first_segment = next_segment;
            remaining_lines -= 1;
        }
        breaks
    }

    /// Adds the given text to the output buffer, keeping track of the current
    /// column, but also processing line returns depending on the user config.
    /// We support optional trimming trailing whitespace and reconstructing
    /// indentation on empty lines between statements.
    fn add_to_output(&mut self, text: &str) {
        for c in text.chars() {
            if c == '\n' {
                self.pending_newlines += 1;
            } else {
                self.add_to_output_process_newlines();
                self.output.push(c);
                if c == '\t' {
                    self.column += self.config.indent_size;
                } else {
                    self.column += 1;
                }
            }
        }
    }

    fn add_to_output_process_newlines(&mut self) {
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

    fn add_to_output_finish(mut self) -> String {
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
