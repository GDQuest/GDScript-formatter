#![allow(clippy::unwrap_used)]
/// Tests that formatting input files in subfolders produces valid, idempotent
/// GDScript output.
///
/// For each input file in the test directories, this test parses the file,
/// generates an intermediate representation, renders it back to text, and
/// verifies that the output parses without error. It also checks that re-parsing
/// the output produces identical results (that formatting is idempotent).
///
/// It's related to the "integration" tests but it does not check the output
/// style of the formatter: When rewriting the formatter I started it as a
/// "no-op" that would parse the code, walk the AST, break it down into
/// intermediate representation, and then re-render it back to text from that
/// intermediate representation. This test battery verified every step of the
/// way that the intermediate representation preserved all the information
/// needed to re-render the code correctly.
use gdscript_formatter::FormatterConfiguration;
use gdscript_formatter::formatter::build_formatter_intermediate_representation;
use gdscript_formatter::node_kind::GDScriptNodeKind;
use gdscript_formatter::parser::ParseInput;
use gdscript_formatter::renderer::{PrinterConfiguration, render};
use std::fs;

test_each_file::test_each_path! { in "./tests/input" => test_ir_format_file }
test_each_file::test_each_path! { in "./tests/reorder_code/input" => test_ir_format_file }

fn test_ir_format_file(file_path: &std::path::Path) {
    let input = fs::read_to_string(file_path)
        .unwrap_or_else(|_| panic!("Failed to read {}", file_path.display()));

    let config = FormatterConfiguration::default();
    let parsed = ParseInput::new(&input, &config)
        .ok_or_else(|| format!("Failed to parse {}", file_path.display()))
        .expect("parse failed");

    let mut render_elements = Vec::new();
    build_formatter_intermediate_representation(&parsed, &mut render_elements);
    let config = PrinterConfiguration::default();
    let mut output = String::new();
    render(&render_elements, &input, &config, &mut output);

    if !parsed.has_parse_errors {
        assert_reparses_cleanly(
            &output,
            file_path,
            "formatted output does not parse cleanly",
        );
    }

    assert_idempotent(
        &input,
        &output,
        &config,
        file_path,
        "formatting is not idempotent",
    );
}

fn assert_reparses_cleanly(output: &str, file_path: &std::path::Path, error_message: &str) {
    let config = FormatterConfiguration::default();
    let parsed = ParseInput::new(output, &config);
    if parsed.is_none() {
        panic!("{} for {}", error_message, file_path.display(),);
    }
    let parsed = parsed.unwrap();
    let root = parsed.tree.root_node();
    if has_error(&root) {
        panic!(
            "{}: output contains ERROR node for {}",
            error_message,
            file_path.display(),
        );
    }
}

fn has_error(node: &tree_sitter::Node) -> bool {
    if GDScriptNodeKind::get_kind_from_ast_node(*node) == GDScriptNodeKind::Error {
        return true;
    }
    let child_count = node.child_count();
    let mut index = 0;
    while index < child_count {
        if let Some(child) = node.child(index as u32) {
            if has_error(&child) {
                return true;
            }
        }
        index += 1;
    }
    false
}

fn assert_idempotent(
    _input: &str,
    first_output: &str,
    config: &PrinterConfiguration,
    file_path: &std::path::Path,
    error_message: &str,
) {
    let format_config = FormatterConfiguration::default();
    let second_parsed = ParseInput::new(first_output, &format_config)
        .ok_or_else(|| {
            format!(
                "Failed to re-parse first output for {}",
                file_path.display()
            )
        })
        .expect("reparse failed");

    let mut second_docs = Vec::new();
    build_formatter_intermediate_representation(&second_parsed, &mut second_docs);
    let mut second_output = String::new();
    render(&second_docs, first_output, config, &mut second_output);

    if first_output != second_output {
        panic!(
            "{} for {}\nfirst output:\n{:?}\nsecond output:\n{:?}",
            error_message,
            file_path.display(),
            first_output,
            second_output,
        );
    }
}
