mod cli;

use std::{
    env, fs,
    io::{self, IsTerminal, Read, Write},
    path::PathBuf,
    thread,
};

use gdscript_formatter::linter::rule_config::{
    get_all_rule_names, parse_disabled_rules, validate_rule_names,
};
use gdscript_formatter::{
    FormatterConfiguration, RenderElement, format_gdscript, format_gdscript_with_buffers,
    linter::LinterConfig,
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
    max_line_length: Option<usize>,
    blank_lines_around_definitions: Option<u16>,
    continuation_indent_level: Option<u16>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args();

    if let Command::Lint {
        disabled_linter_rules,
        max_line_length,
        do_list_rules,
        do_pretty_print,
    } = args.command
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
            max_line_length,
        };

        let input_gdscript_files = find_gdscript_files(&args.input_file_paths)?;
        return run_linter(&input_gdscript_files, linter_config, do_pretty_print);
    }

    let Command::Format {
        do_print_to_stdout,
        do_check_formatted_only,
        use_spaces,
        indent_size,
        use_safe_mode,
        do_reorder_code,
        max_line_length,
        blank_lines_around_definitions,
        continuation_indent_level,
    } = args.command
    else {
        unreachable!();
    };

    let mut config = FormatterConfiguration::default();

    config.printer.indent_size = indent_size;
    config.printer.use_spaces = use_spaces;
    config.safe = use_safe_mode;
    config.reorder_code = do_reorder_code;

    let config_overrides = FormatterConfigOverrides {
        max_line_length,
        blank_lines_around_definitions,
        continuation_indent_level,
    };

    if args.input_file_paths.is_empty() && !io::stdin().is_terminal() {
        let mut input_content = String::new();
        io::stdin()
            .read_to_string(&mut input_content)
            .map_err(|error| format!("Failed to read from stdin: {}", error))?;

        let formatted_content = format_gdscript(&input_content, &config)?;

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

    let input_paths = if args.input_file_paths.is_empty() {
        vec![
            env::current_dir()
                .map_err(|error| format!("Failed to get current directory: {}", error))?,
        ]
    } else {
        args.input_file_paths
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
    do_pretty_print: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut linter = gdscript_formatter::linter::GDScriptLinter::new(config)?;
    let has_issues = linter.lint_files(input_files, do_pretty_print)?;

    if has_issues {
        std::process::exit(1);
    }

    Ok(())
}

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
    gdscript_formatter::editorconfig::apply_editorconfig_to_formatter_config(
        &mut file_config,
        file_path,
    );
    if let Some(max_line_length) = config_overrides.max_line_length {
        file_config.printer.max_line_length = max_line_length;
    }
    if let Some(blank_lines_around_definitions) = config_overrides.blank_lines_around_definitions {
        file_config.blank_lines_around_definitions = blank_lines_around_definitions;
    }
    if let Some(continuation_indent_level) = config_overrides.continuation_indent_level {
        file_config.printer.continuation_indent_level = continuation_indent_level;
    }

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
