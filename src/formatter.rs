use std::io::BufWriter;

use regex::RegexBuilder;
use topiary_core::{formatter, Language, Operation, TopiaryQuery};

use crate::FormatterConfig;

pub fn format_gdscript(content: &str) -> Result<String, Box<dyn std::error::Error>> {
    format_gdscript_with_config(content, &FormatterConfig::default())
}

pub fn format_gdscript_with_config(
    content: &str,
    config: &FormatterConfig,
) -> Result<String, Box<dyn std::error::Error>> {
    let query = include_str!("../queries/gdscript.scm");

    let indent_string = if config.use_spaces {
        " ".repeat(config.indent_size)
    } else {
        "\t".to_string()
    };

    let language: Language = Language {
        name: "gdscript".to_owned(),
        query: TopiaryQuery::new(&tree_sitter_gdscript::LANGUAGE.into(), query).unwrap(),
        grammar: tree_sitter_gdscript::LANGUAGE.into(),
        indent: Some(indent_string),
    };

    let preprocessed_content = preprocess(content);

    let mut input = preprocessed_content.as_bytes();
    let mut output = Vec::new();
    let mut writer = BufWriter::new(&mut output);

    let formatter_result = formatter(
        &mut input,
        &mut writer,
        &language,
        Operation::Format {
            skip_idempotence: true,
            tolerate_parsing_errors: true,
        },
    );

    if let Err(formatter_error) = formatter_result {
        return Err(format!("Topiary formatting failed: {}", formatter_error).into());
    }

    drop(writer);

    let formatted_content = String::from_utf8(output)
        .map_err(|e| format!("Failed to parse topiary output as UTF-8: {}", e))?;

    Ok(formatted_content)
}

fn preprocess(content: &str) -> String {
    let mut content = content.to_owned();

    content = remove_newlines_after_extends_statement(content);

    content
}

/// Remove additional new line characters after `extends_statement`
fn remove_newlines_after_extends_statement(mut content: String) -> String {
    // This regex matches substrings which:
    // - must NOT contain "#" or "\n" characters between new line and "extends" keyword
    // - must end with at least one new line character
    // - must contain `extends_name` character sequence that satisfies one of the following conditions:
    //   - consists out of alphanumeric characters
    //   - consists out of any characters (except new lines) between double quotes
    let re = RegexBuilder::new(
        r#"(?P<extends_line>^[^#\n]*extends )(?P<extends_name>([a-zA-Z0-9]+|".*?"))\n(\n*)"#,
    )
    .multi_line(true)
    .build()
    .expect("regex should compile");
    content = re
        .replace(&content, "$extends_line$extends_name\n")
        .to_string();
    content
}
