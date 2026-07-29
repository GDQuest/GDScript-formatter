//! GDScript code formatter following the official style guide (with some
//! configuration options).
//!
//! Call [format_gdscript] to format GDScript code on a single file without
//! having to do memory allocations yourself. For batch formatting, use
//! [format_gdscript_with_buffers] to reuse pre-allocated buffers.
//!
//! It takes source code and a `FormatterConfig` struct and returns the
//! formatted string. Internally it runs a sequence of three broad steps: the
//! parser produces a syntax tree (uses the GDScript tree-sitter parser we
//! maintain), the formatter builds an intermediate representation of the
//! formatted code, and the renderer produces the final string.
//!
//! If you turn safe mode on, the output is reparsed and an error is returned
//! if it contains syntax errors. Use this to prevent formatting errors.

pub mod editorconfig;
pub mod formatter;
pub mod linter;
pub mod node_kind;
pub mod parser;
pub mod renderer;
pub mod reorder;
pub mod verify_structure;

pub use renderer::{PrinterConfiguration, RenderElement};

/// Selects which delimiters the formatter prefers for string literals.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuoteStyle {
    /// Keep the delimiters from the source code.
    Preserve,
    /// Prefer single quote delimiters.
    Single,
    /// Prefer double quote delimiters.
    Double,
}

impl QuoteStyle {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "preserve" => Some(Self::Preserve),
            "single" => Some(Self::Single),
            "double" => Some(Self::Double),
            _ => None,
        }
    }
}

/// Holds all the formatter configuration. The printer field stores the max line
/// width, indent, and any other rendering config the renderer needs. `safe` and
/// future formatter-only feature flags should be added here.
#[derive(Clone)]
pub struct FormatterConfiguration {
    pub printer: PrinterConfiguration,
    pub safe: bool,
    pub reorder_code: bool,
    /// Number of blank lines around top-level function and inner class
    /// declarations. We apply 2 by default following the GDScript style guide,
    /// set this to 1 to reduce the number of blank lines.
    pub blank_lines_around_definitions: u16,
    /// If set to `single` or `double`, the formatter will try to use that quote
    /// style for strings.
    pub quote_style: QuoteStyle,
}

impl Default for FormatterConfiguration {
    fn default() -> Self {
        Self {
            printer: PrinterConfiguration::default(),
            safe: false,
            reorder_code: false,
            blank_lines_around_definitions: 2,
            quote_style: QuoteStyle::Preserve,
        }
    }
}

/// Convenience wrapper around `format_gdscript_with_buffers`. Use it to format
/// a single GDScript file without doing memory allocations yourself.
///
/// For formatting multiple files, prefer [format_gdscript_with_buffers] to
/// reuse pre-allocated buffers across multiple calls.
pub fn format_gdscript(source: &str, config: &FormatterConfiguration) -> Result<String, String> {
    let mut render_elements = Vec::new();
    let mut output = String::new();
    format_gdscript_with_buffers(source, config, &mut render_elements, &mut output)?;
    Ok(output)
}

/// Format GDScript source code into pre-allocated `render_elements` and
/// `output` buffers. Reuse the same buffers across repeated calls to avoid
/// re-allocation when formatting many files.
pub fn format_gdscript_with_buffers(
    source: &str,
    config: &FormatterConfiguration,
    render_elements: &mut Vec<RenderElement>,
    output: &mut String,
) -> Result<(), String> {
    let parsed = parser::ParseInput::new(source, config)
        .ok_or_else(|| "Failed to parse input".to_string())?;
    formatter::build_formatter_intermediate_representation(&parsed, render_elements);

    // The renderer clamps every blank-line run to `maximum_blank_lines`. If a
    // user configures more blank lines around definitions than that cap
    // allows, the separator the formatter emits between declarations would be
    // silently truncated back down. Raise the cap to match so the configured
    // value is always honored.
    let mut printer_config = config.printer.clone();
    if printer_config.maximum_blank_lines < config.blank_lines_around_definitions {
        printer_config.maximum_blank_lines = config.blank_lines_around_definitions;
    }
    renderer::render(render_elements, source, &printer_config, output);

    if config.safe {
        let reparsed = parser::ParseInput::new(output, config)
            .ok_or_else(|| "Verify structure: formatted output does not parse".to_string())?;
        if !verify_structure::trees_structurally_equal(
            &parsed.tree,
            &reparsed.tree,
            parsed.kind_lookup,
        ) {
            return Err(
                "Verify structure: formatted output is structurally different from input. \
                 Keeping original source."
                    .to_string(),
            );
        }
    }

    Ok(())
}
