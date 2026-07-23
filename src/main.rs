//! Command-line entry point for the GDScript formatter and linter.
//!
//! Formatting settings come from three places. The formatter starts with its
//! built-in defaults, applies the `.editorconfig` properties that match each
//! input file, and then applies any CLI option flags explicitly provided on the
//! command line. Command line flags override editorconfig settings which
//! override built-in defaults.
//!
//! Stdin input uses the current directory to find `.editorconfig`. It uses a
//! synthetic `stdin.gd` path so filename sections such as `[*.gd]` also match.
//!
//! The formatter keeps both batch implementations below. The sequential path
//! is the normal path because it is easier to follow in stack traces. The
//! parallel path remains available for performance-sensitive callers and uses
//! the same per-file configuration logic.

mod cli;

use std::{
    env, fs,
    io::{self, IsTerminal, Read, Write},
    path::{Path, PathBuf},
    thread,
};

use gdscript_formatter::linter::rule_config::{
    get_all_rule_names, parse_disabled_rules, validate_rule_names,
};
use gdscript_formatter::{
    FormatterConfiguration, QuoteStyle, RenderElement, format_gdscript,
    format_gdscript_with_buffers, linter::LinterConfig,
};
use std::collections::HashSet;

use cli::{Command, parse_args};

const ERROR_CODE_NOT_FORMATTED: i32 = 1;

#[derive(Debug, Clone)]
struct FormatterOutput {
    index: usize,
    file_path: PathBuf,
    formatted_content: String,
    is_formatted: bool,
}

#[derive(Clone, Copy)]
struct FormatterConfigOverrides {
    /// Explicitly requested tab or space indentation.
    use_spaces: Option<bool>,
    /// Explicitly requested indentation width.
    indent_size: Option<usize>,
    /// Explicitly requested maximum line length.
    max_line_length: Option<usize>,
    /// Explicitly requested blank lines between top-level definitions.
    blank_lines_around_definitions: Option<u16>,
    /// Explicitly requested continuation indentation depth.
    continuation_indent_level: Option<u16>,
    /// Explicitly requested string quote style.
    quote_style: Option<QuoteStyle>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parsed_cli_args = parse_args();

    if let Command::Lint {
        disabled_linter_rules,
        max_line_length,
        do_list_rules,
        do_pretty_print,
    } = parsed_cli_args.command
    {
        if do_list_rules {
            println!("Available linting rules:");
            for rule in get_all_rule_names() {
                println!("  {}", rule);
            }
            return Ok(());
        }

        let disabled_rules = if let Some(disable_str) = disabled_linter_rules {
            let rules = parse_disabled_rules(&disable_str);
            if let Err(invalid_rules) = validate_rule_names(&rules) {
                eprintln!("Error: Invalid rule names: {}", invalid_rules.join(", "));
                eprintln!("Use --list-rules to see all available rules");
                std::process::exit(1);
            }
            rules
        } else {
            HashSet::new()
        };

        let linter_config = LinterConfig {
            disabled_rules,
            max_line_length: max_line_length.unwrap_or(100),
        };

        let input_gdscript_files = find_gdscript_files(&parsed_cli_args.input_file_paths)?;
        return run_linter(
            &input_gdscript_files,
            linter_config,
            max_line_length,
            do_pretty_print,
        );
    }

    let Command::Format {
        do_print_to_stdout,
        do_check_formatted_only,
        use_spaces,
        indent_size,
        use_verify_structure,
        do_reorder_code,
        max_line_length,
        blank_lines_around_definitions,
        continuation_indent_level,
        quote_style,
    } = parsed_cli_args.command
    else {
        unreachable!();
    };

    let mut config = FormatterConfiguration {
        safe: use_verify_structure,
        reorder_code: do_reorder_code,
        ..Default::default()
    };

    if let Some(quote_style) = quote_style {
        config.quote_style = quote_style;
    }

    let config_overrides = FormatterConfigOverrides {
        use_spaces,
        indent_size,
        max_line_length,
        blank_lines_around_definitions,
        continuation_indent_level,
        quote_style,
    };

    if parsed_cli_args.input_file_paths.is_empty() && !io::stdin().is_terminal() {
        let mut input_content = String::new();
        io::stdin()
            .read_to_string(&mut input_content)
            .map_err(|error| format!("Failed to read from stdin: {}", error))?;

        let mut stdin_config = config.clone();
        let current_directory = env::current_dir().expect("Failed to get current directory");
        // When running from stdin users would still like to apply editorconfig
        // settings. For the most part, you'd use a section like `[*.gd]`
        // matching GDScript files. we fake running the formatter on a `.gd`
        // file in the current directory to get user settings to apply.
        config_apply_editorconfig_then_cli_overrides(
            &mut stdin_config,
            &current_directory.join("stdin.gd"),
            config_overrides,
        );
        let formatted_content = format_gdscript(&input_content, &stdin_config)?;

        if do_check_formatted_only {
            if input_content != formatted_content {
                eprintln!("The input passed via stdin is not formatted");
                std::process::exit(1);
            } else {
                eprintln!("The input passed via stdin is already formatted");
            }
        } else {
            print!("{}", formatted_content);
        }

        return Ok(());
    }

    let input_paths = if parsed_cli_args.input_file_paths.is_empty() {
        vec![
            env::current_dir()
                .map_err(|error| format!("Failed to get current directory: {}", error))?,
        ]
    } else {
        parsed_cli_args.input_file_paths
    };
    let input_gdscript_files = find_gdscript_files(&input_paths)?;

    let total_files = input_gdscript_files.len();

    eprint!(
        "Formatting {} file{}...",
        total_files,
        if total_files == 1 { "" } else { "s" }
    );
    let _ = io::stdout().flush();

    let mut sorted_outputs: Vec<Result<FormatterOutput, String>> =
        format_files_parallel(&input_gdscript_files, &config, config_overrides);

    sorted_outputs.sort_by(compare_output_index);

    let mut all_formatted = true;
    let mut modified_file_count = 0;
    let mut unformatted_files = Vec::new();
    for output in sorted_outputs {
        match output {
            Ok(output) => {
                if do_check_formatted_only {
                    if !output.is_formatted {
                        all_formatted = false;
                        unformatted_files.push(output.file_path);
                    }
                } else if do_print_to_stdout {
                    terminal_clear_line();
                    eprint!("\r");
                    if total_files > 1 {
                        println!("#--file:{}", output.file_path.display());
                    }
                    print!("{}", output.formatted_content);
                } else if !output.is_formatted {
                    fs::write(&output.file_path, output.formatted_content).map_err(|error| {
                        format!(
                            "Failed to write to file {}: {}",
                            output.file_path.display(),
                            error
                        )
                    })?;
                    modified_file_count += 1;
                }
            }
            Err(error_msg) => {
                return Err(error_msg.into());
            }
        }
    }

    if do_check_formatted_only {
        terminal_clear_line();
        if all_formatted {
            eprintln!("\rAll {} file(s) are formatted", total_files);
        } else {
            eprintln!("\rSome files are not formatted");
            for file_path in unformatted_files {
                eprintln!("{}", file_path.display());
            }
            std::process::exit(ERROR_CODE_NOT_FORMATTED);
        }
    } else if !do_print_to_stdout {
        terminal_clear_line();
        if total_files == 1 {
            if modified_file_count > 0 {
                eprintln!("\rFormatted {}", input_gdscript_files[0].display());
            } else {
                eprintln!("\rAlready formatted: {}", input_gdscript_files[0].display());
            }
        } else {
            let already_formatted_count = total_files - modified_file_count;
            if modified_file_count > 0 && already_formatted_count > 0 {
                eprintln!(
                    "\rFormatted {} files, {} already formatted",
                    modified_file_count, already_formatted_count
                );
            } else if modified_file_count > 0 {
                eprintln!("\rFormatted {} files", modified_file_count);
            } else {
                eprintln!("\rAll {} files already formatted", total_files);
            }
        }
    }

    Ok(())
}

fn run_linter(
    input_files: &[PathBuf],
    config: LinterConfig,
    max_line_length_override: Option<usize>,
    do_pretty_print: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut linter = gdscript_formatter::linter::GDScriptLinter::new(config)?;
    let has_issues = linter.lint_files_with_editorconfig(
        input_files,
        do_pretty_print,
        max_line_length_override,
    )?;

    if has_issues {
        std::process::exit(1);
    }

    Ok(())
}

/// Formats one file after applying its matching editorconfig settings.
///
/// Different files can fall under different editorconfig settings so we clone
/// the base configuration and apply file-specific settings individually.
/// CLI options override editorconfig are applied last by [`apply_file_config`].
fn format_one_file(
    index: usize,
    file_path: &PathBuf,
    config: &FormatterConfiguration,
    config_overrides: FormatterConfigOverrides,
    render_elements: &mut Vec<RenderElement>,
    output: &mut String,
) -> Result<FormatterOutput, String> {
    let input_content = fs::read_to_string(file_path)
        .map_err(|error| format!("Failed to read file {}: {}", file_path.display(), error))?;

    // We need to clone that config because files in nested directories can
    // match different EditorConfig files and rules.
    let mut file_config = config.clone();
    config_apply_editorconfig_then_cli_overrides(&mut file_config, file_path, config_overrides);
    format_gdscript_with_buffers(&input_content, &file_config, render_elements, output)
        .map_err(|error| format!("Failed to format file {}: {}", file_path.display(), error))?;

    let is_formatted = input_content == *output;

    Ok(FormatterOutput {
        index,
        file_path: file_path.clone(),
        formatted_content: output.clone(),
        is_formatted,
    })
}

/// Applies project editorconfig settings first and CLI settings second.
///
/// The override fields are `Option`s because `None` means that the user did
/// not pass that flag. Without this distinction, a CLI default such as an
/// indentation size of four would look like an explicit request and would
/// incorrectly override `.editorconfig`.
fn config_apply_editorconfig_then_cli_overrides(
    config: &mut FormatterConfiguration,
    config_path: &Path,
    config_overrides: FormatterConfigOverrides,
) {
    gdscript_formatter::editorconfig::apply_editorconfig_to_formatter_config(config, config_path);
    if let Some(use_spaces) = config_overrides.use_spaces {
        config.printer.use_spaces = use_spaces;
    }
    if let Some(indent_size) = config_overrides.indent_size {
        config.printer.indent_size = indent_size;
    }
    if let Some(max_line_length) = config_overrides.max_line_length {
        config.printer.max_line_length = max_line_length;
    }
    if let Some(blank_lines_around_definitions) = config_overrides.blank_lines_around_definitions {
        config.blank_lines_around_definitions = blank_lines_around_definitions;
    }
    if let Some(continuation_indent_level) = config_overrides.continuation_indent_level {
        config.printer.continuation_indent_level = continuation_indent_level;
    }
    if let Some(quote_style) = config_overrides.quote_style {
        config.quote_style = quote_style;
    }
}

/// Formats files concurrently but preserves their original order when
/// outputting the results.
fn format_files_parallel(
    files: &[PathBuf],
    config: &FormatterConfiguration,
    config_overrides: FormatterConfigOverrides,
) -> Vec<Result<FormatterOutput, String>> {
    if files.is_empty() {
        return Vec::new();
    }

    let hardware_threads = match thread::available_parallelism() {
        Ok(n) => n.get(),
        Err(_) => 1,
    };
    let thread_count = hardware_threads.min(files.len());
    let chunk_size = files.len().div_ceil(thread_count);

    thread::scope(|scope| {
        let mut handles = Vec::with_capacity(thread_count);
        for (chunk_index, chunk) in files.chunks(chunk_size).enumerate() {
            let handle = scope.spawn(move || {
                format_chunk(chunk, chunk_index, chunk_size, config, config_overrides)
            });
            handles.push(handle);
        }

        let mut all = Vec::with_capacity(files.len());
        for handle in handles {
            all.extend(handle.join().expect("worker thread panicked"));
        }
        all
    })
}

fn format_chunk(
    chunk: &[PathBuf],
    chunk_index: usize,
    chunk_size: usize,
    config: &FormatterConfiguration,
    config_overrides: FormatterConfigOverrides,
) -> Vec<Result<FormatterOutput, String>> {
    let mut results = Vec::with_capacity(chunk.len());
    let mut render_elements: Vec<RenderElement> = Vec::new();
    let mut output = String::new();
    for (local_index, file_path) in chunk.iter().enumerate() {
        let global_index = chunk_index * chunk_size + local_index;
        results.push(format_one_file(
            global_index,
            file_path,
            config,
            config_overrides,
            &mut render_elements,
            &mut output,
        ));
    }
    results
}

fn find_gdscript_files(
    input_paths: &[PathBuf],
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut gdscript_file_paths = Vec::new();
    let mut paths_to_check: Vec<PathBuf> = Vec::with_capacity(input_paths.len());
    for path in input_paths {
        paths_to_check.push(path.to_path_buf());
    }

    while let Some(current_path) = paths_to_check.pop() {
        if current_path.is_dir() {
            let entries = fs::read_dir(&current_path).map_err(|error| {
                format!(
                    "Failed to read directory {}: {}",
                    current_path.display(),
                    error
                )
            })?;
            for entry in entries {
                let entry = entry.map_err(|error| {
                    format!(
                        "Failed to read entry in {}: {}",
                        current_path.display(),
                        error
                    )
                })?;
                if entry.path().is_dir() {
                    paths_to_check.push(entry.path());
                } else if let Some(extension) = entry.path().extension() {
                    if extension == "gd" {
                        gdscript_file_paths.push(entry.path());
                    }
                }
            }
        } else if let Some(extension) = current_path.extension() {
            if extension == "gd" {
                gdscript_file_paths.push(current_path);
            }
        }
    }

    gdscript_file_paths.sort();
    gdscript_file_paths.dedup();

    if gdscript_file_paths.is_empty() {
        eprintln!(
            "Error: No GDScript files found in the arguments provided. Please provide at least one .gd file or directory containing .gd files."
        );
        std::process::exit(1);
    }

    Ok(gdscript_file_paths)
}

fn compare_output_index(
    left: &Result<FormatterOutput, String>,
    right: &Result<FormatterOutput, String>,
) -> std::cmp::Ordering {
    let left_index = match left {
        Ok(formatter_output) => formatter_output.index,
        Err(_) => usize::MAX,
    };
    let right_index = match right {
        Ok(formatter_output) => formatter_output.index,
        Err(_) => usize::MAX,
    };
    left_index.cmp(&right_index)
}

fn terminal_clear_line() {
    eprint!("\r{}", " ".repeat(80));
}
