//! This module pairs the source string with the parsed tree-sitter GDScript
//! AST. The other layers in the formatter receive this struct to read the AST
//! and determine the code's formatting. Across the formatter we try hard to
//! only read text and only copy it when it's time to render the formatted code.

use crate::QuoteStyle;
use crate::node_kind::GDScriptNodeKind;
use tree_sitter;

pub struct ParseInput<'src> {
    pub source: &'src str,
    pub tree: tree_sitter::Tree,
    pub kind_lookup: &'static [GDScriptNodeKind; 256],
    pub reorder_code: bool,
    pub blank_lines_around_definitions: u16,
    /// Extra indent level for continuation lines (default 2).
    pub continuation_indent_level: u16,
    pub quote_style: QuoteStyle,
    /// Byte ranges where formatting is disabled by `# fmt: off` / `# fmt: on`
    /// marker comments. Sorted, non-overlapping.
    pub disabled_regions: Vec<RegionWithDisabledFormatting>,
}

/// This is a byte range where formatting is disabled (i.e. by a pair of
/// #fmt:off and #fmt:on comments).
///
/// start is the index of the fmt:off's # character, and end is the index of the
/// end of fmt:on's own text (not including the trailing newline, matching how
/// every other node's end byte is tracked elsewhere in the formatter).
///
/// Defining the range like that allows us to reuse
/// RenderElement::UnformattedSource for this feature: the renderer inserts
/// newlines and indents before the starting #, and everything from there on is
/// copied as-is from the source file.
#[derive(Clone, Copy)]
pub struct RegionWithDisabledFormatting {
    pub start: usize,
    pub end: usize,
}

/// Scans the source code character by character to find pairs of "# fmt: off"
/// and "# fmt: on" comments. Returns the byte ranges where formatting
/// should be disabled.
fn find_disabled_regions(source: &str) -> Vec<RegionWithDisabledFormatting> {
    // Nathan: I tried first to iterate over the tree sitter node tree. I don't
    // know if I was doing something wrong, but it made the entire formatter
    // ~20% slower. There is also the option of querying nodes with tree sitter,
    // but tree sitter queries compile at runtime and that compilation is orders
    // of magnitude slower.
    struct Scanner<'a> {
        bytes: &'a [u8],
        current_index: usize,
    }

    impl<'a> Scanner<'a> {
        fn new(source: &'a str) -> Self {
            Self {
                bytes: source.as_bytes(),
                current_index: 0,
            }
        }

        fn is_at_end(&self) -> bool {
            self.current_index >= self.bytes.len()
        }

        fn peek(&self) -> Option<u8> {
            self.bytes.get(self.current_index).copied()
        }

        fn advance(&mut self) {
            if !self.is_at_end() {
                self.current_index += 1;
            }
        }

        fn skip_whitespace(&mut self) {
            while let Some(byte) = self.peek() {
                if byte.is_ascii_whitespace() {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        /// If `text_bytes` is found from the current position, advances the
        /// scanner past it and returns true. Otherwise returns false without
        /// moving.
        fn match_byte_string_literal(&mut self, text_bytes: &[u8]) -> bool {
            let end = self.current_index + text_bytes.len();
            if end <= self.bytes.len() && self.bytes[self.current_index..end] == *text_bytes {
                self.current_index = end;
                true
            } else {
                false
            }
        }

        fn advance_to_end_of_line(&mut self) {
            while !self.is_at_end() && self.peek() != Some(b'\n') {
                self.advance();
            }
        }

        /// Returns the index of the newline character at the end of the current
        /// line (or the EOF if this is the last line), without advancing.
        fn find_end_of_line(&self) -> usize {
            let mut search_position = self.current_index;
            while search_position < self.bytes.len() && self.bytes[search_position] != b'\n' {
                search_position += 1;
            }
            search_position
        }
    }

    /// Try to skip over a string at the current position. Returns true if a
    /// string was consumed (or an unterminated string skipped to the end of the
    /// file).
    ///
    /// GDScript has three kinds of string marks: double quote, single quote, or
    /// triple quote for multiline strings. Anything that is escaped is ignored.
    fn try_skip_string(scanner: &mut Scanner) -> bool {
        let saved = scanner.current_index;

        if scanner.match_byte_string_literal(b"\"\"\"") {
            while !scanner.is_at_end() {
                if scanner.match_byte_string_literal(b"\"\"\"") {
                    return true;
                }
                if scanner.match_byte_string_literal(b"\\") {
                    scanner.advance();
                    continue;
                }
                scanner.advance();
            }
            return true;
        }

        for &quote_sign in b"\"'" {
            if scanner.match_byte_string_literal(&[quote_sign]) {
                while !scanner.is_at_end() {
                    if scanner.match_byte_string_literal(b"\\") {
                        scanner.advance();
                        continue;
                    }
                    if scanner.match_byte_string_literal(&[quote_sign]) {
                        return true;
                    }
                    scanner.advance();
                }
                return true;
            }
        }

        scanner.current_index = saved;
        false
    }

    let mut regions: Vec<RegionWithDisabledFormatting> = Vec::new();
    let mut disabled_region_start: Option<usize> = None;
    let mut scanner = Scanner::new(source);

    while !scanner.is_at_end() {
        if try_skip_string(&mut scanner) {
            continue;
        }

        if !scanner.match_byte_string_literal(b"#") {
            scanner.advance();
            continue;
        }

        // The region start is the position of the '#' character.
        let hash_position = scanner.current_index - 1;

        scanner.skip_whitespace();

        if !scanner.match_byte_string_literal(b"fmt") {
            scanner.advance_to_end_of_line();
            continue;
        }

        scanner.skip_whitespace();

        if !scanner.match_byte_string_literal(b":") {
            scanner.advance_to_end_of_line();
            continue;
        }

        scanner.skip_whitespace();

        let is_off = scanner.match_byte_string_literal(b"off");
        let is_on = !is_off && scanner.match_byte_string_literal(b"on");

        if !is_off && !is_on {
            scanner.advance_to_end_of_line();
            continue;
        }

        let comment_end = scanner.find_end_of_line();

        if is_off {
            if disabled_region_start.is_none() {
                disabled_region_start = Some(hash_position);
            }
        } else if let Some(start_of_region) = disabled_region_start.take() {
            regions.push(RegionWithDisabledFormatting {
                start: start_of_region,
                end: comment_end,
            });
        }

        // Advance to newline character to continue scanning from there.
        scanner.current_index = comment_end;
        scanner.advance();
    }

    if let Some(region_start) = disabled_region_start {
        regions.push(RegionWithDisabledFormatting {
            start: region_start,
            end: source.len(),
        });
    }

    regions
}

impl<'src> ParseInput<'src> {
    pub fn new(source: &'src str, config: &crate::FormatterConfiguration) -> Option<Self> {
        let mut parser = tree_sitter::Parser::new();
        let language = tree_sitter_gdscript::LANGUAGE.into();
        parser
            .set_language(&language)
            .expect("tree_sitter_gdscript::LANGUAGE is a build-time invariant");
        let tree = parser.parse(source.as_bytes(), None)?;
        let kind_lookup = GDScriptNodeKind::populate_lookup_table();
        let disabled_regions = find_disabled_regions(source);
        Some(Self {
            source,
            tree,
            kind_lookup,
            reorder_code: config.reorder_code,
            blank_lines_around_definitions: config.blank_lines_around_definitions,
            continuation_indent_level: config.printer.continuation_indent_level,
            quote_style: config.quote_style,
            disabled_regions,
        })
    }
}
