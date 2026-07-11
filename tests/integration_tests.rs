#![allow(clippy::unwrap_used)]
/// Runs the formatter over a series of input files and verifies the output
/// matches the expected output file. See files in the ./input and ./expected
/// folders.
use gdscript_formatter::linter::{GDScriptLinter, LinterConfig};
use gdscript_formatter::{FormatterConfiguration, format_gdscript};
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
