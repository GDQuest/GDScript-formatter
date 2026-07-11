//! Reads .editorconfig files and applies any values found there into the
//! formatter config.
//!
//! This reads both standard keys and some custom ones we use to extend
//! editorconfig's features. Custom keys are read as plain strings and parsed
//! manually.

use crate::FormatterConfiguration;
use ec4rs::property::{FinalNewline, IndentSize, IndentStyle, MaxLineLen, TrimTrailingWs};
use std::path::Path;

pub fn apply_editorconfig_to_formatter_config(
    config: &mut FormatterConfiguration,
    editorconfig_file_path: &Path,
) {
    let properties = match ec4rs::properties_of(editorconfig_file_path) {
        Ok(mut properties) => {
            properties.use_fallbacks();
            properties
        }
        Err(_) => return,
    };

    if let Ok(IndentStyle::Spaces) = properties.get::<IndentStyle>() {
        config.printer.use_spaces = true;
    } else if let Ok(IndentStyle::Tabs) = properties.get::<IndentStyle>() {
        config.printer.use_spaces = false;
    }
    if let Ok(IndentSize::Value(size)) = properties.get::<IndentSize>() {
        if size > 0 {
            config.printer.indent_size = size;
        }
    }

    if let Ok(MaxLineLen::Value(max_line_length)) = properties.get::<MaxLineLen>() {
        if max_line_length > 0 {
            config.printer.max_line_length = max_line_length;
        }
    }

    if let Ok(FinalNewline::Value(insert_final_newline)) = properties.get::<FinalNewline>() {
        config.printer.insert_final_newline = insert_final_newline;
    }

    if let Ok(TrimTrailingWs::Value(trim_trailing_whitespace)) = properties.get::<TrimTrailingWs>()
    {
        config.printer.trim_trailing_whitespace = trim_trailing_whitespace;
    }

    // These keys are custom to this program and not part of the standard
    // editorconfig keys, but according to the editor config specification, all
    // values are read which allows us to add custom key value pairs.
    let raw = properties.get_raw_for_key("gdscript_formatter_blank_lines_around_definitions");
    if let Some(found_value) = raw.into_option() {
        if let Ok(found_number) = found_value.parse::<u16>() {
            config.blank_lines_around_definitions = found_number;
        }
    }

    let raw = properties.get_raw_for_key("gdscript_formatter_continuation_indent_level");
    if let Some(found_value) = raw.into_option() {
        if let Ok(found_number) = found_value.parse::<u16>() {
            config.printer.continuation_indent_level = found_number;
        }
    }

    let raw = properties.get_raw_for_key("gdscript_formatter_indent_blank_lines");
    if let Some(found_value) = raw.into_option() {
        if found_value == "true" {
            config.printer.indent_blank_lines = true;
        } else if found_value == "false" {
            config.printer.indent_blank_lines = false;
        }
    }
}
