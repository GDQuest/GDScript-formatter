#![allow(clippy::unwrap_used)]
/// Runs the formatter with reorder_code enabled on each input file. Verifies
/// that the output matches the expected output file. Similar to
/// integration_tests, but focuses on reordering code.
use gdscript_formatter::{FormatterConfiguration, format_gdscript};
use std::fs;

test_each_file::test_each_path! { in "./tests/reorder_code/input" => test_reorder }

fn test_reorder(file_path: &std::path::Path) {
    let input = fs::read_to_string(file_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", file_path.display(), e));

    let config = FormatterConfiguration {
        reorder_code: true,
        ..Default::default()
    };

    let output = format_gdscript(&input, &config)
        .unwrap_or_else(|e| panic!("Failed to format {}: {}", file_path.display(), e));

    let expected_path = file_path.to_str().unwrap().replace("/input/", "/expected/");
    let expected = fs::read_to_string(&expected_path)
        .unwrap_or_else(|e| panic!("Failed to read expected {}: {}", expected_path, e));

    assert_eq!(
        output,
        expected,
        "reorder output differs from expected for {}",
        file_path.display()
    );
}
