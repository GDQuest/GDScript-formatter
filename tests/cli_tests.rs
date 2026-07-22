use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn test_directory() -> std::path::PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after the Unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "gdscript-formatter-cli-test-{}-{}",
        std::process::id(),
        timestamp
    ));
    fs::create_dir(&path).expect("should create temporary test directory");
    path
}

fn formatter_command(directory: &std::path::Path, args: &[&str]) -> Command {
    let binary = std::env::current_exe()
        .expect("test executable path should be available")
        .parent()
        .expect("test executable should have a parent")
        .parent()
        .expect("test executable should be in target/debug/deps")
        .join("gdscript-formatter");
    let mut command = Command::new(binary);
    command.current_dir(directory).args(args);
    command
}

#[test]
fn stdin_and_file_modes_apply_editorconfig_and_cli_overrides() {
    let directory = test_directory();
    fs::write(
        directory.join(".editorconfig"),
        "root = true\n\n[*.gd]\nindent_style = space\nindent_size = 8\nmax_line_length = 120\ngdscript_formatter_blank_lines_around_definitions = 2\ngdscript_formatter_quote_style = double\n",
    )
    .expect("should write EditorConfig");

    let mut stdin_command = formatter_command(
        &directory,
        &[
            "--blank-lines-around-definitions",
            "1",
            "--use-spaces",
            "--indent-size",
            "2",
            "--quote-style",
            "single",
        ],
    );

    let input = "func first():\n\tvar value = \"first\"\n\n\nfunc second():\n\tpass\n";
    let expected = "func first():\n  var value = 'first'\n\nfunc second():\n  pass\n";

    stdin_command.stdin(Stdio::piped()).stdout(Stdio::piped());
    let mut child = stdin_command.spawn().expect("should start formatter");
    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(input.as_bytes())
        .expect("should write formatter input");
    let stdin_output = child
        .wait_with_output()
        .expect("should collect formatter output");
    assert!(stdin_output.status.success());
    assert_eq!(
        String::from_utf8(stdin_output.stdout).expect("stdin output should be valid UTF-8"),
        expected,
    );

    let input_path = directory.join("input.gd");
    fs::write(&input_path, input).expect("should write input file");
    let file_output = formatter_command(
        &directory,
        &[
            "--stdout",
            "--blank-lines-around-definitions",
            "1",
            "--use-spaces",
            "--indent-size",
            "2",
            "--quote-style",
            "single",
            "input.gd",
        ],
    )
    .output()
    .expect("should format file");
    assert!(file_output.status.success());
    assert_eq!(
        String::from_utf8(file_output.stdout).expect("file output should be valid UTF-8"),
        expected,
    );

    let lint_path = directory.join("lint.gd");
    fs::write(&lint_path, format!("#{}\n", "1".repeat(119))).expect("should write lint input");
    let lint_output = formatter_command(&directory, &["lint", "lint.gd"])
        .output()
        .expect("should lint file");
    assert!(lint_output.status.success());
    assert!(lint_output.stdout.is_empty());

    let lint_override_output =
        formatter_command(&directory, &["lint", "--max-line-length", "100", "lint.gd"])
            .output()
            .expect("should lint file with an override");
    assert!(!lint_override_output.status.success());
    assert!(
        String::from_utf8(lint_override_output.stdout)
            .expect("lint output should be valid UTF-8")
            .contains("maximum allowed is 100")
    );

    fs::remove_dir_all(directory).expect("should remove temporary test directory");
}
