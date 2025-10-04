use std::{
    env, fs,
    io::{self, IsTerminal, Read, Write},
    path::PathBuf,
};

use clap::{CommandFactory, Parser};
use rayon::prelude::*;

use gdscript_formatter::linter::rule_config::{
    get_all_rule_names, parse_disabled_rules, validate_rule_names,
};
use gdscript_formatter::{
    FormatterConfig, formatter::format_gdscript_with_config, linter::LinterConfig,
};
use std::collections::HashSet;

/// This struct is used to hold all the information about the result when
/// formatting a single file. Now that we use parallel processing, we need to
/// keep track of the original index to order the files in the output when
/// printing results.
#[derive(Debug, Clone)]
struct FormatterOutput {
    index: usize,
    file_path: PathBuf,
    formatted_content: String,
    is_formatted: bool,
}

#[derive(Parser)]
/// A GDScript formatter following the official style guide.
///
/// This program formats GDScript files with a consistent style and indentation
/// using Topiary and Tree-sitter.
///
/// By default, the formatter overwrites input files with the formatted code.
/// Use the --stdout flag to output to standard output instead.
///
/// The latest version of the GDScript style guide can be found at:
/// https://docs.godotengine.org/en/stable/tutorials/scripting/gdscript/gdscript_styleguide.html
#[clap(
    // Use the version number directly from Cargo.toml at compile time
    version = env!("CARGO_PKG_VERSION"),
    max_term_width = 120
)]
struct Args {
    /// The GDScript file(s) to format. If no file paths are provided, the
    /// program reads from standard input and outputs to standard output.
    #[arg(value_name = "FILES")]
    input: Vec<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,

    /// Output formatted code to stdout without changing FILES.
    ///
    /// If multiple input files are provided, each file's content is preceded
    /// by the line "#--file:<file_path>".
    ///
    /// This flag is ignored when reading from stdin since stdout is always
    /// used.
    #[arg(long)]
    stdout: bool,

    /// Check if FILES are formatted, making no changes.
    ///
    /// Exits with code 0 if the file is already formatted and 1 if it's not
    /// formatted.
    #[arg(short, long)]
    check: bool,

    /// Use spaces for indentation instead of tabs.
    ///
    /// Use --indent-size to set the number of spaces to use as indentation.
    #[arg(long)]
    use_spaces: bool,

    /// Set how many spaces to use for indentation.
    ///
    /// Has no effect without the --use-spaces flag.
    #[arg(long, default_value = "4", value_name = "NUM")]
    indent_size: usize,

    /// Reorder code to follow the official GDScript style guide.
    ///
    /// Reorder source-level declarations (signals, properties, methods, etc.)
    /// in this order: signals, enums, constants, properties, static and built-in
    /// virtual methods, public methods, pseudo-private methods, and sub-classes.
    ///
    /// If enabled, reordering happens after formatting the code.
    #[arg(long)]
    reorder_code: bool,

    /// Enable safe mode.
    ///
    /// This mode ensures that after formatting, the code still has the same
    /// syntax and structure as when initially parsed. If not, formatting is
    /// canceled.
    ///
    /// This offers a good amount protection against the formatter failing
    /// on new syntax at the cost of a small little extra running time.
    /// Currently incompatible with --reorder-code.
    ///
    /// WARNING: this is not a perfect solution. Some rare edge cases may still
    /// lead to syntax changes.
    #[arg(short, long, conflicts_with = "reorder_code")]
    safe: bool,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Lint GDScript files for style and convention issues
    Lint {
        #[arg(help = "Input GDScript file(s) to lint", value_name = "FILES")]
        input: Vec<PathBuf>,
        #[arg(
            long,
            help = "Disable specific linting rules (comma-separated)",
            value_name = "RULES"
        )]
        disable: Option<String>,
        #[arg(long, help = "Maximum line length allowed", default_value = "100")]
        max_line_length: usize,
        #[arg(long, help = "List all available linting rules")]
        list_rules: bool,
        #[arg(long, help = "Use pretty formatting for lint output")]
        pretty: bool,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // If there are no arguments and nothing piped from stdin, print the help message
    if env::args().len() == 1 && io::stdin().is_terminal() {
        Args::command().print_help()?;
        println!();
        return Ok(());
    }

    let args = Args::parse();

    // Handle lint subcommand
    if let Some(Commands::Lint {
        input,
        disable,
        max_line_length,
        list_rules,
        pretty,
    }) = args.command
    {
        if list_rules {
            println!("Available linting rules:");
            for rule in get_all_rule_names() {
                println!("  {}", rule);
            }
            return Ok(());
        }

        let disabled_rules = if let Some(disable_str) = disable {
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

        return run_linter(input, linter_config, pretty);
    }

    let config = FormatterConfig {
        indent_size: args.indent_size,
        use_spaces: args.use_spaces,
        reorder_code: args.reorder_code,
        safe: args.safe,
    };

    if args.input.is_empty() {
        let mut input_content = String::new();
        io::stdin()
            .read_to_string(&mut input_content)
            .map_err(|error| format!("Failed to read from stdin: {}", error))?;

        let formatted_content = format_gdscript_with_config(&input_content, &config)?;

        if args.check {
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

    let input_gdscript_files: Vec<&PathBuf> = args
        .input
        .iter()
        .filter(|path| path.extension().map_or(false, |ext| ext == "gd"))
        .collect();

    if input_gdscript_files.is_empty() {
        eprintln!(
            "Error: No GDScript files found in the arguments provided. Please provide at least one .gd file."
        );
        std::process::exit(1);
    }

    let total_files = input_gdscript_files.len();

    eprint!(
        "Formatting {} file{}...",
        total_files,
        if total_files == 1 { "" } else { "s" }
    );
    io::stdout().flush().unwrap();

    // We use the rayon library to automatically process files in parallel for
    // us. The formatter runs largely single threaded so this speeds things up a
    // lot on multi-core CPUs
    let outputs: Vec<Result<FormatterOutput, String>> = input_gdscript_files
        .par_iter()
        .enumerate()
        .map(|(index, file_path)| {
            let input_content = fs::read_to_string(file_path).map_err(|error| {
                format!("Failed to read file {}: {}", file_path.display(), error)
            })?;

            let formatted_content =
                format_gdscript_with_config(&input_content, &config).map_err(|error| {
                    format!("Failed to format file {}: {}", file_path.display(), error)
                })?;

            let is_formatted = input_content == formatted_content;

            Ok(FormatterOutput {
                index,
                file_path: (*file_path).clone(),
                formatted_content,
                is_formatted,
            })
        })
        .collect();

    // Restore the original order of the input files based on their initial index
    let mut sorted_outputs: Vec<_> = outputs.into_iter().collect();
    sorted_outputs.sort_by_key(|output| {
        match output {
            Ok(output) => output.index,
            // Sort errors at the end in no particular order
            Err(_) => usize::MAX,
        }
    });

    // If true, all input files were already formatted (used for check mode)
    let mut all_formatted = true;
    for output in sorted_outputs {
        match output {
            Ok(output) => {
                if args.check {
                    if !output.is_formatted {
                        all_formatted = false;
                    }
                } else if args.stdout {
                    // Clear the progress message before printing formatted files to stdout
                    terminal_clear_line();
                    // A little bit hacky, but because terminals by default output both stdout and stderr
                    // we need to return carriage to the start to print formatted output from the start of the line
                    eprint!("\r");
                    // If there are multiple input files we still allow stdout but we print a separator
                    if total_files > 1 {
                        println!("#--file:{}", output.file_path.display());
                    }
                    print!("{}", output.formatted_content);
                } else {
                    fs::write(&output.file_path, output.formatted_content).map_err(|e| {
                        format!(
                            "Failed to write to file {}: {}",
                            output.file_path.display(),
                            e
                        )
                    })?;
                }
            }
            Err(error_msg) => {
                return Err(error_msg.into());
            }
        }
    }

    if args.check {
        if all_formatted {
            terminal_clear_line();
            eprintln!("\rAll {} file(s) are formatted", total_files);
        } else {
            terminal_clear_line();
            eprintln!("\rSome files are not formatted");
            std::process::exit(1);
        }
    } else if !args.stdout {
        terminal_clear_line();
        if total_files == 1 {
            eprintln!("\rFormatted {}", input_gdscript_files[0].display());
        } else {
            eprintln!("\rFormatted {} files", total_files);
        }
    }

    Ok(())
}

fn run_linter(
    input_files: Vec<PathBuf>,
    config: LinterConfig,
    pretty: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut linter = gdscript_formatter::linter::GDScriptLinter::new(config)?;
    let has_issues = linter.lint_files(input_files, pretty)?;

    if has_issues {
        std::process::exit(1);
    }

    Ok(())
}

fn terminal_clear_line() {
    eprint!("\r{}", " ".repeat(80));
}
