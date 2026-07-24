//! Command-line argument parser for the GDScript formatter.
//!
//! At the time of writing, the linter program that ships with the formatter and
//! runs as a sub-command. It may be moved to a dedicated program in the future.
//!
//! NB: do not replace with a dependency like clap: it brings too many
//! dependencies only to save a little straightforward code.
use gdscript_formatter::QuoteStyle;
use std::path::PathBuf;

const HELP_FORMATTER: &str = "\
	A GDScript formatter following the official style guide.

	Usage: gdscript-formatter [OPTIONS] [FILES]...
	       gdscript-formatter lint [OPTIONS] [FILES]...

	Arguments:
	  <FILES>...  GDScript files or directories to format. If empty, uses
	              the current directory. Reads from stdin when piped.

	Options:
	  -c, --check                                Check if files are formatted, exit 1 if not
	      --verify-structure                     Verify formatted output has the same structure as the input
	      --stdout                               Write to stdout instead of overwriting files
	      --use-spaces                           Use spaces instead of tabs for indentation
	      --indent-size <NUM>                    Spaces per indent level (default: 4)
	      --reorder-code                         Reorder code to match the style guide
	      --max-line-length <NUM>                Maximum line length before wrapping (default: 100)
	      --blank-lines-around-definitions <NUM> Blank lines between top-level definitions (default: 2)
	      --continuation-indent-level <NUM>      Extra indent for line continuations (default: 2)
	      --quote-style <STYLE>                  String quotes: preserve, single, or double (default: preserve)
	  -h, --help                                 Print help
	  -V, --version                              Print version

	Subcommands:
	  lint                     Lint GDScript files for style issues

	Run 'gdscript-formatter lint --help' for lint options.
";

const HELP_LINTER: &str = "\
Lint GDScript files for style and convention issues.

Usage: gdscript-formatter lint [OPTIONS] [FILES]...

Arguments:
  <FILES>...                 GDScript files or directories to lint

Options:
      --disable <RULES>       Disable specific rules (comma-separated)
      --max-line-length <NUM>  Maximum line length allowed (default: 100)
      --list-rules            List all available linting rules
      --pretty                Use pretty formatting for lint output
  -h, --help                  Print help
";

/// Represents the parsed command-line arguments for the GDScript formatter. You
/// can use exactly one command: currently, Format (the default) or Lint.
pub struct CliArguments {
    /// List of input file paths or directories to process.
    pub input_file_paths: Vec<PathBuf>,
    /// Which command to run.
    pub command: Command,
}

/// The command selected by the user on the command line, and its associated
/// arguments.
pub enum Command {
    /// Format GDScript files according to the official style guide.
    Format {
        /// If true, prints the formatted output to stdout instead of writing to
        /// files.
        do_print_to_stdout: bool,
        /// If true, only checks if the files are formatted, without modifying
        /// them. Returns error code ERROR_CODE_NOT_FORMATTED if any of the input
        /// files are not formatted.
        do_check_formatted_only: bool,
        /// If set, uses spaces for indentation instead of tabs.
        use_spaces: Option<bool>,
        /// Number of spaces to use for indentation.
        indent_size: Option<usize>,
        /// If true, the formatter will re-parse the formatted code and verify it
        /// has the same structure as the original before writing it to files.
        use_verify_structure: bool,
        /// If true, the formatter will reorder code to follow the official
        /// GDScript style guide's recommended order of code elements.
        do_reorder_code: bool,
        /// Maximum line length before wrapping. If not specified, uses an
        /// editorconfig value, or falls back to a default of 100 characters.
        max_line_length: Option<usize>,
        /// Number of blank lines between top-level definitions (functions,
        /// inner classes). 2 by default.
        blank_lines_around_definitions: Option<u16>,
        /// Extra indent level for continuation lines.
        continuation_indent_level: Option<u16>,
        /// If set to `single` or `double`, the formatter will try to use that
        /// quote style for strings.
        quote_style: Option<QuoteStyle>,
    },
    /// Lint GDScript files for style and convention issues.
    Lint {
        /// Optional list of comma-separated linter rule names to disable.
        disabled_linter_rules: Option<String>,
        /// Maximum line length for the linter.
        max_line_length: Option<usize>,
        /// If true, the linter program will list all available linting rules and
        /// exit.
        do_list_rules: bool,
        /// If true, the linter program will print the list of available linting
        /// rules in a human-readable format.
        do_pretty_print: bool,
    },
}

/// Internal discriminator used during parsing to track which command's flags
/// we are currently parsing.
enum ActiveCommand {
    Format,
    Lint,
}

pub fn parse_args() -> CliArguments {
    let argument_list: Vec<String> = std::env::args().collect();
    let mut active_command = ActiveCommand::Format;

    let mut input_file_paths: Vec<PathBuf> = Vec::new();
    let mut format_do_print_to_stdout = false;
    let mut format_do_check_formatted_only = false;
    let mut format_use_spaces: Option<bool> = None;
    let mut format_indent_size: Option<usize> = None;
    let mut format_use_verify_structure = false;
    let mut format_do_reorder_code = false;
    let mut format_max_line_length: Option<usize> = None;
    let mut format_blank_lines_around_definitions: Option<u16> = None;
    let mut format_continuation_indent_level: Option<u16> = None;
    let mut format_quote_style: Option<QuoteStyle> = None;

    let mut lint_disabled_rules: Option<String> = None;
    let mut lint_max_line_length: Option<usize> = None;
    let mut lint_list_rules = false;
    let mut lint_pretty_print = false;

    // The first positional argument optionally selects a command. If it is
    // "lint", we run the linter program. Defaults to the formatter.
    let mut current_argument_index = 1;
    if argument_list.len() > 1 && argument_list[1] == "lint" {
        active_command = ActiveCommand::Lint;
        current_argument_index = 2;
    }

    while current_argument_index < argument_list.len() {
        let current_argument = argument_list[current_argument_index].as_str();

        // -- stops option parsing. Everything after it is a file path.
        if current_argument == "--" {
            current_argument_index += 1;
            while current_argument_index < argument_list.len() {
                let file_path = argument_list[current_argument_index].as_str();
                input_file_paths.push(PathBuf::from(file_path));
                current_argument_index += 1;
            }
            break;
        }

        if current_argument == "--help" || current_argument == "-h" {
            match active_command {
                ActiveCommand::Format => print!("{}", HELP_FORMATTER),
                ActiveCommand::Lint => print!("{}", HELP_LINTER),
            }
            std::process::exit(0);
        }

        if current_argument == "--version" || current_argument == "-V" {
            println!("gdscript-formatter {}", env!("CARGO_PKG_VERSION"));
            std::process::exit(0);
        }

        if let Some(flag_without_prefix) = current_argument.strip_prefix("--") {
            let (flag_name, assigned_value) = split_flag_and_value(flag_without_prefix);
            match active_command {
                ActiveCommand::Format => match flag_name {
                    "stdout" => {
                        require_no_value(assigned_value, "--stdout");
                        format_do_print_to_stdout = true;
                    }
                    "check" => {
                        require_no_value(assigned_value, "--check");
                        format_do_check_formatted_only = true;
                    }
                    "use-spaces" => {
                        require_no_value(assigned_value, "--use-spaces");
                        format_use_spaces = Some(true);
                    }
                    "verify-structure" => {
                        require_no_value(assigned_value, "--verify-structure");
                        format_use_verify_structure = true;
                    }
                    "safe" => {
                        // DEPRECATED: We keep this flag for anyone who already
                        // used it as part of a CI or of their workflow. But
                        // it's now replaced by --verify-structure, which is a
                        // more accurate name for this mode.
                        require_no_value(assigned_value, "--safe");
                        format_use_verify_structure = true;
                    }
                    "reorder-code" => {
                        require_no_value(assigned_value, "--reorder-code");
                        format_do_reorder_code = true;
                    }
                    "indent-size" => {
                        let value = consume_flag_value(
                            assigned_value,
                            &argument_list,
                            &mut current_argument_index,
                            "--indent-size",
                        );
                        format_indent_size = match value.parse::<usize>() {
                            Ok(n) => Some(n),
                            Err(_) => print_error_invalid_argument(&format!(
                                "--indent-size expects a number, got '{}'",
                                value
                            )),
                        };
                    }
                    "max-line-length" => {
                        let value = consume_flag_value(
                            assigned_value,
                            &argument_list,
                            &mut current_argument_index,
                            "--max-line-length",
                        );
                        format_max_line_length = match value.parse::<usize>() {
                            Ok(n) => Some(n),
                            Err(_) => print_error_invalid_argument(&format!(
                                "--max-line-length expects a number, got '{}'",
                                value
                            )),
                        };
                    }
                    "blank-lines-around-definitions" => {
                        let value = consume_flag_value(
                            assigned_value,
                            &argument_list,
                            &mut current_argument_index,
                            "--blank-lines-around-definitions",
                        );
                        format_blank_lines_around_definitions = match value.parse::<u16>() {
                            Ok(n) => Some(n),
                            Err(_) => print_error_invalid_argument(&format!(
                                "--blank-lines-around-definitions expects a number, got '{}'",
                                value
                            )),
                        };
                    }
                    "continuation-indent-level" => {
                        let value = consume_flag_value(
                            assigned_value,
                            &argument_list,
                            &mut current_argument_index,
                            "--continuation-indent-level",
                        );
                        format_continuation_indent_level = match value.parse::<u16>() {
                            Ok(n) => Some(n),
                            Err(_) => print_error_invalid_argument(&format!(
                                "--continuation-indent-level expects a number, got '{}'",
                                value
                            )),
                        };
                    }
                    "quote-style" => {
                        let value = consume_flag_value(
                            assigned_value,
                            &argument_list,
                            &mut current_argument_index,
                            "--quote-style",
                        );
                        format_quote_style = match QuoteStyle::from_name(&value) {
                            Some(quote_style) => Some(quote_style),
                            None => print_error_invalid_argument(&format!(
                                "--quote-style expects preserve, single, or double, got '{}'",
                                value
                            )),
                        };
                    }
                    _ => print_error_invalid_argument(&format!(
                        "unexpected argument '--{}'",
                        flag_name
                    )),
                },
                ActiveCommand::Lint => match flag_name {
                    "disable" => {
                        let value = consume_flag_value(
                            assigned_value,
                            &argument_list,
                            &mut current_argument_index,
                            "--disable",
                        );
                        lint_disabled_rules = Some(value);
                    }
                    "max-line-length" => {
                        let value = consume_flag_value(
                            assigned_value,
                            &argument_list,
                            &mut current_argument_index,
                            "--max-line-length",
                        );
                        lint_max_line_length = match value.parse::<usize>() {
                            Ok(n) => Some(n),
                            Err(_) => print_error_invalid_argument(&format!(
                                "--max-line-length expects a number, got '{}'",
                                value
                            )),
                        };
                    }
                    "list-rules" => {
                        require_no_value(assigned_value, "--list-rules");
                        lint_list_rules = true;
                    }
                    "pretty" => {
                        require_no_value(assigned_value, "--pretty");
                        lint_pretty_print = true;
                    }
                    _ => print_error_invalid_argument(&format!(
                        "unexpected argument '--{}'",
                        flag_name
                    )),
                },
            }
        } else if current_argument.starts_with('-') && current_argument.len() > 1 {
            let short_flags = &current_argument[1..];
            for flag_char in short_flags.chars() {
                match flag_char {
                    'c' => {
                        if matches!(active_command, ActiveCommand::Lint) {
                            print_error_invalid_argument("unexpected argument '-c'");
                        }
                        format_do_check_formatted_only = true;
                    }
                    's' => {
                        if matches!(active_command, ActiveCommand::Lint) {
                            print_error_invalid_argument("unexpected argument '-s'");
                        }
                        format_use_verify_structure = true;
                    }
                    _ => print_error_invalid_argument(&format!(
                        "unexpected argument '-{}'",
                        flag_char
                    )),
                }
            }
        } else {
            input_file_paths.push(PathBuf::from(current_argument));
        }
        current_argument_index += 1;
    }

    match active_command {
        ActiveCommand::Format => CliArguments {
            input_file_paths,
            command: Command::Format {
                do_print_to_stdout: format_do_print_to_stdout,
                do_check_formatted_only: format_do_check_formatted_only,
                use_spaces: format_use_spaces,
                indent_size: format_indent_size,
                use_verify_structure: format_use_verify_structure,
                do_reorder_code: format_do_reorder_code,
                max_line_length: format_max_line_length,
                blank_lines_around_definitions: format_blank_lines_around_definitions,
                continuation_indent_level: format_continuation_indent_level,
                quote_style: format_quote_style,
            },
        },
        ActiveCommand::Lint => CliArguments {
            input_file_paths,
            command: Command::Lint {
                disabled_linter_rules: lint_disabled_rules,
                max_line_length: lint_max_line_length,
                do_list_rules: lint_list_rules,
                do_pretty_print: lint_pretty_print,
            },
        },
    }
}

fn split_flag_and_value(flag: &str) -> (&str, Option<&str>) {
    let separator_position = flag.find('=');
    match separator_position {
        Some(position) => (&flag[..position], Some(&flag[position + 1..])),
        None => (flag, None),
    }
}

/// Resolves the value for a flag that takes an argument.
///
/// If the value was already assigned using an equal sign (e.g. `--flag=value`),
/// returns it directly. Otherwise, advances `current_scan_position` and
/// consumes the next command-line argument as the value.
///
/// Errors out if the flag is the last argument on the command line and no value
/// could be found.
fn consume_flag_value(
    assigned_value: Option<&str>,
    argument_list: &[String],
    current_argument_index: &mut usize,
    flag_name: &str,
) -> String {
    if let Some(value) = assigned_value {
        return value.to_string();
    }
    *current_argument_index += 1;
    if *current_argument_index >= argument_list.len() {
        print_error_invalid_argument(&format!("{} requires a value", flag_name));
    }
    argument_list[*current_argument_index].clone()
}

/// Ensures that a flag was not given a value. Exits with an error if a value
/// was assigned. This is used for flags that do not take an argument.
fn require_no_value(assigned_value: Option<&str>, flag_name: &str) {
    if assigned_value.is_some() {
        print_error_invalid_argument(&format!("{} does not take a value", flag_name));
    }
}

fn print_error_invalid_argument(message: &str) -> ! {
    eprintln!("gdscript-formatter: error: {}", message);
    std::process::exit(2);
}
