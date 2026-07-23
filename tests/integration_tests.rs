#![allow(clippy::unwrap_used)]
/// Runs the formatter over a series of input files and verifies the output
/// matches the expected output file. See files in the ./input and ./expected
/// folders.
use gdscript_formatter::linter::{GDScriptLinter, LinterConfig};
use gdscript_formatter::{
    FormatterConfiguration, PrinterConfiguration, QuoteStyle, format_gdscript,
};
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::path::Path;

test_each_file::test_each_path! { in "./tests/input" => test_file }
test_each_file::test_each_path! { in "./tests/lint/input" as lint => test_lint_file  }

fn make_whitespace_visible(s: &str) -> String {
    s.replace(' ', "·")
        .replace('\t', "⇥   ")
        .replace('\n', "↲\n")
}

fn assert_formatted_eq(
    result: &str,
    expected: &str,
    file_path: &Path,
    error_context_message: &str,
) {
    if result != expected {
        eprintln!("\n{} - {}", error_context_message, file_path.display());
        eprintln!("Diff between expected(-) and actual output(+):");
        let diff = TextDiff::from_lines(expected, result);
        for change in diff.iter_all_changes() {
            let text = make_whitespace_visible(&change.to_string());
            match change.tag() {
                ChangeTag::Insert => eprint!("\x1B[92m+{}\x1B[0m", text),
                ChangeTag::Delete => eprint!("\x1B[91m-{}\x1B[0m", text),
                ChangeTag::Equal => eprint!(" {}", text),
            }
        }
        panic!("Assertion failed: {}", error_context_message);
    }
}

fn test_file(file_path: &Path) {
    test_file_with_config(file_path, &FormatterConfiguration::default(), true);
}

fn test_lint_file(file_path: &Path) {
    let file_name = file_path.file_name().expect("path is not a file path");
    let file_stem = file_path.file_stem().expect("path is not a file path");

    let input_path = file_path;
    let expected_path = file_path
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("expected/")
        .join(format!("{}.txt", file_stem.to_string_lossy()));

    let input_content = fs::read_to_string(input_path)
        .unwrap_or_else(|_| panic!("Failed to read {}", input_path.display()));
    let expected_content = fs::read_to_string(&expected_path)
        .unwrap_or_else(|_| panic!("Failed to read {}", expected_path.display()));

    let mut linter = GDScriptLinter::new(LinterConfig::default())
        .unwrap_or_else(|_| panic!("Failed to create linter for {}", input_path.display()));
    let issues = linter
        .lint(&input_content, &input_path.to_string_lossy())
        .unwrap_or_else(|_| panic!("Failed to lint {}", input_path.display()));

    // Format issues as they would appear in the CLI output
    let mut actual_output = String::new();
    for issue in issues {
        let relative_path = format!("tests/lint/input/{}", file_name.to_string_lossy());
        actual_output.push_str(&format!(
            "{}:{}:{}:{}: {}\n",
            relative_path,
            issue.line,
            issue.rule,
            match issue.severity {
                gdscript_formatter::linter::LintSeverity::Error => "error",
                gdscript_formatter::linter::LintSeverity::Warning => "warning",
            },
            issue.message
        ));
    }

    if actual_output.ends_with('\n') {
        actual_output.pop();
    }

    assert_eq!(
        actual_output.trim(),
        expected_content.trim(),
        "Lint output for {} doesn't match expected",
        file_name.to_string_lossy()
    );
}

fn test_file_with_config(
    file_path: &Path,
    config: &FormatterConfiguration,
    check_idempotence: bool,
) {
    let file_name = file_path.file_name().expect("path is not a file path");

    let input_path = file_path;
    let expected_path = file_path
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("expected/")
        .join(file_name);

    let input_content = fs::read_to_string(input_path)
        .unwrap_or_else(|_| panic!("Failed to read {}", input_path.display()));
    let expected_content = fs::read_to_string(&expected_path)
        .unwrap_or_else(|_| panic!("Failed to read {}", expected_path.display()));

    let result = format_gdscript(&input_content, config)
        .unwrap_or_else(|_| panic!("Failed to format {}", input_path.display()));

    assert_formatted_eq(
        &result,
        &expected_content,
        input_path,
        "First formatting output doesn't match expected",
    );

    if check_idempotence {
        let second_result = format_gdscript(&result, config)
            .unwrap_or_else(|_| panic!("Failed to format {}", input_path.display()));
        assert_formatted_eq(
            &second_result,
            &result,
            input_path,
            "Idempotence check failed, formatting a second time gave different results",
        );
    }
}

#[test]
fn quote_style_changes_compatible_string_literals() {
    let input = r#"var double = "double"
var single = 'already single'
var escaped = "line\n"
var contains_preferred_quote = "don't change"
var multiline = """multiple
lines"""
var multiline_with_preferred_quote = """has ''' inside"""
var string_name = &"Name"
var node_path = ^"Node/Path"
"#;
    let expected = r#"var double = 'double'
var single = 'already single'
var escaped = 'line\n'
var contains_preferred_quote = "don't change"
var multiline = '''multiple
lines'''
var multiline_with_preferred_quote = """has ''' inside"""
var string_name = &'Name'
var node_path = ^'Node/Path'
"#;
    let config = FormatterConfiguration {
        quote_style: QuoteStyle::Single,
        safe: true,
        ..Default::default()
    };

    let output = format_gdscript(input, &config).unwrap();
    assert_eq!(output, expected);
    assert_eq!(format_gdscript(&output, &config).unwrap(), output);
}

#[test]
fn quote_style_prefers_double_quotes() {
    let input = r#"var single = 'single'
var contains_preferred_quote = 'keep "double"'
var multiline = '''multiple
lines'''
"#;
    let expected = r#"var single = "single"
var contains_preferred_quote = 'keep "double"'
var multiline = """multiple
lines"""
"#;
    let config = FormatterConfiguration {
        quote_style: QuoteStyle::Double,
        ..Default::default()
    };

    assert_eq!(format_gdscript(input, &config).unwrap(), expected);
}

#[test]
fn parse_errors_disable_reordering_without_disabling_formatting() {
    let input = "var b=1\nvar name.bla = value\nvar a=2\n";
    let config = FormatterConfiguration {
        reorder_code: true,
        ..Default::default()
    };

    let output = format_gdscript(input, &config).unwrap();

    assert_eq!(output, "var b = 1\nvar name.bla = value\nvar a = 2\n");
}

#[test]
fn generic_type_parameters_never_break() {
    // Type-level generic parameters like Dictionary[String, String] must
    // stay on one line even when max_line_length would otherwise force a
    // break. Splitting the brackets produces invalid GDScript.
    let input = "func test() -> Dictionary[String, String]:\n    return {}\n";
    let config = FormatterConfiguration {
        printer: PrinterConfiguration {
            max_line_length: 10,
            ..Default::default()
        },
        ..Default::default()
    };
    let output = format_gdscript(input, &config).unwrap();
    // The generic type must remain on a single line.
    let expected_line_0 = "func test() -> Dictionary[String, String]:";
    assert!(
        output.lines().next() == Some(expected_line_0),
        "Expected first line to be '{}' but got '{}'",
        expected_line_0,
        output.lines().next().unwrap_or("")
    );
}

#[test]
fn editorconfig_applies_quote_style() {
    let mut config = FormatterConfiguration::default();
    gdscript_formatter::editorconfig::apply_editorconfig_to_formatter_config(
        &mut config,
        Path::new("tests/manual_test_files/hello_world.gd"),
    );

    assert_eq!(config.quote_style, QuoteStyle::Single);
}
