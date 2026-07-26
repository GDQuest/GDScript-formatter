use godot::classes::class_macros::private::virtuals::ZipReader::{
    Array, PackedStringArray,
};
use godot::prelude::{
    Base, Dictionary, ExtensionLibrary, GodotClass, GString, Object, Variant,
    gdextension, godot_api, godot_error,
};
use gdscript_formatter::{
    FormatterConfiguration, PrinterConfiguration, QuoteStyle, format_gdscript,
};
use gdscript_formatter::linter::{
    LintIssue, LintSeverity, LinterConfig, lint_gdscript_with_config,
};

struct FormatterExtension;

#[gdextension]
unsafe impl ExtensionLibrary for FormatterExtension {}

#[derive(GodotClass)]
#[class(init, singleton, base=Object)]
struct GDScriptFormatter {
    base: Base<Object>,
}

#[godot_api]
impl GDScriptFormatter {
    #[func]
    pub fn format_gdscript(
        &self,
        source: GString,
        config: Dictionary<Variant, Variant>,
    ) -> GString {
        let formatter_config = dict_to_formatter_config(&config);
        match format_gdscript(&source.to_string(), &formatter_config) {
            Ok(formatted) => GString::from(&formatted),
            Err(error) => {
                godot_error!("Formatter error: {}", error);
                GString::new()
            },
        }
    }

    #[func]
    pub fn lint_gdscript(
        &self,
        source: GString,
        config: Dictionary<Variant, Variant>,
    ) -> Array<Dictionary<Variant, Variant>> {
        let linter_config = dict_to_linter_config(&config);
        match lint_gdscript_with_config(
                &source.to_string(), "", &linter_config) {
            Ok(issues) => issues_to_array(&issues),
            Err(error) => {
                godot_error!("Linter error: {}", error);
                Array::<Dictionary<Variant, Variant>>::new()
            },
        }
    }
}

macro_rules! extract_field {
    ($dict:expr, $key:ident, $type:ty, $target:expr) => {
        if let Some(variant) = $dict.get(stringify!($key)) {
            match variant.try_to::<$type>() {
                Ok(val) => $target.$key = val,
                Err(error) => godot_error!(
                        "Config '{}' is invalid: {}", stringify!($key), error),
            }
        }
    };
    // Special case for usize
    ($dict:expr, $key:ident, $type:ty, as usize, $target:expr) => {
        if let Some(variant) = $dict.get(stringify!($key)) {
            match variant.try_to::<$type>() {
                Ok(val) => $target.$key = val as usize,
                Err(error) => godot_error!(
                        "Config '{}' is invalid: {}", stringify!($key), error),
            }
        }
    };
}

fn dict_to_formatter_config(
    dict: &Dictionary<Variant, Variant>,
) -> FormatterConfiguration {
    let mut result = FormatterConfiguration::default();
    extract_field!(dict, safe, bool, result);
    extract_field!(dict, reorder_code, bool, result);
    extract_field!(dict, blank_lines_around_definitions, u16, result);
    if let Some(variant) = dict.get("printer") {
        match variant.try_to::<Dictionary<Variant, Variant>>() {
            Ok(sub_dict) => result.printer = dict_to_printer_config(&sub_dict),
            Err(error) =>
                godot_error!("Config 'printer' is invalid: {}", error),
        }
    }
    if let Some(variant) = dict.get("quote_style") {
        match variant.try_to::<GString>().ok()
                .and_then(|gstr| QuoteStyle::from_name(&gstr.to_string())) {
            Some(valid_style) => result.quote_style = valid_style,
            None => godot_error!("Config 'quote_style' is invalid"),
        }
    }
    result
}

fn dict_to_printer_config(
    dict: &Dictionary<Variant, Variant>,
) -> PrinterConfiguration {
    let mut result = PrinterConfiguration::default();
    extract_field!(dict, max_line_length, i64, as usize, result);
    extract_field!(dict, indent_size, i64, as usize, result);
    extract_field!(dict, use_spaces, bool, result);
    extract_field!(dict, insert_final_newline, bool, result);
    extract_field!(dict, trim_trailing_whitespace, bool, result);
    extract_field!(dict, indent_blank_lines, bool, result);
    extract_field!(dict, maximum_blank_lines, u16, result);
    extract_field!(dict, continuation_indent_level, u16, result);
    result
}

fn dict_to_linter_config(
    dict: &Dictionary<Variant, Variant>,
) -> LinterConfig {
    let mut result = LinterConfig::default();
    extract_field!(dict, max_line_length, i64, as usize, result);
    if let Some(variant) = dict.get("disabled_rules") {
        match variant.try_to::<PackedStringArray>() {
            Ok(rules) =>
                result.disabled_rules = rules.as_slice()
                    .iter()
                    .map(|gd_str| gd_str.to_string())
                    .collect(),
            Err(error) =>
                godot_error!("Config 'disabled_rules' is invalid: {}", error),
        }
    }
    result
}

fn issues_to_array(
    issues: &Vec<LintIssue>,
) -> Array<Dictionary<Variant, Variant>> {
    let mut gd_issues = Array::<Dictionary<Variant, Variant>>::new();
    for issue in issues {
        let mut gd_issue = Dictionary::<Variant, Variant>::new();
        gd_issue.set("line", issue.line as i64);
        gd_issue.set("column", issue.column as i64);
        gd_issue.set("rule", &GString::from(&issue.rule));
        gd_issue.set("severity", match issue.severity {
            LintSeverity::Error => "error",
            LintSeverity::Warning => "warning",
        });
        gd_issue.set("message", &GString::from(&issue.message));
        gd_issues.push(&gd_issue);
    }
    gd_issues
}
