use std::fs;
use std::io::{self, Write};
use std::process::{Command, exit};

fn print_error_and_exit(message: &str) -> ! {
    eprintln!("ERROR: {}", message);
    exit(1);
}

/// Prompts the user for input and returns the trimmed string
fn ask_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().expect("Failed to flush stdout");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read user input");
    return input.trim().to_owned();
}

/// Runs a shell command with the given arguments.
/// Exits the process with an error message if the command fails to start or returns a non-zero exit code.
fn run(cmd: &str, args: &[&str]) {
    let status = Command::new(cmd)
        .args(args)
        .status()
        .unwrap_or_else(|_| print_error_and_exit(&format!("Failed to run {}", cmd)));

    if !status.success() {
        print_error_and_exit(&format!("{} {} failed", cmd, args.join(" ")));
    }
}

/// Parses a "semantic" version number from a string and returns the version above
fn bump_tag_version(current: &str, bump_type: &str) -> Option<String> {
    let mut parts = current.split('.');
    let major = parts.next()?.parse::<u32>().ok()?;
    let minor = parts.next()?.parse::<u32>().ok()?;
    let patch = parts.next()?.parse::<u32>().ok()?;

    match bump_type {
        "major" => Some(format!("{}.0.0", major + 1)),
        "minor" => Some(format!("{}.{}.0", major, minor + 1)),
        "patch" => Some(format!("{}.{}.{}", major, minor, patch + 1)),
        _ => None,
    }
}

fn extract_changelog_section(changelog: &str, version: &str) -> Option<String> {
    let header = format!("## Release {}", version);
    let start = changelog.find(&header)?;
    let section = &changelog[start..];
    let end = section
        .match_indices("\n## Release ")
        .next()
        .map(|(index, _)| index)
        .unwrap_or(section.len());
    Some(section[..end].trim_end().to_owned())
}

fn main() {
    println!("GDScript Formatter release script\n");

    let cargo_toml = fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");

    let current_version = cargo_toml
        .lines()
        .find(|line| line.starts_with("version = "))
        .and_then(|line| line.split('"').nth(1))
        .expect("Failed to find version in Cargo.toml")
        .to_owned();

    println!("Current version: {}", current_version);

    let bump_type = ask_user_input("\nVersion bump type? [major/minor/patch]: ").to_lowercase();
    let new_version = bump_tag_version(&current_version, &bump_type).unwrap_or_else(|| {
        print_error_and_exit("Invalid version or bump type. Use major, minor, or patch.");
    });

    println!("\nNew version will be: {}", new_version);

    let tag_exists = Command::new("git")
        .args([
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("refs/tags/{}", new_version),
        ])
        .status()
        .map(|status| status.success())
        .unwrap_or(false);

    if tag_exists {
        print_error_and_exit(&format!("Git tag '{}' already exists", new_version));
    }
    println!("Git tag does not exist");

    println!("\nCommits since {}:", current_version);
    let commits_output = Command::new("git")
        .args(["log", "--oneline", &format!("{}..HEAD", current_version)])
        .output()
        .unwrap_or_else(|_| print_error_and_exit("Failed to run git log"));
    if !commits_output.status.success() {
        print_error_and_exit("git log failed");
    }
    let commits = String::from_utf8_lossy(&commits_output.stdout).into_owned();
    if commits.trim().is_empty() {
        println!("(no commits since last release)");
    } else {
        print!("{}", commits);
    }
    println!();

    let changelog_path = "CHANGELOG.md";
    let expected_header = format!("## Release {}", new_version);
    let mut changelog = fs::read_to_string(changelog_path).expect("Failed to read CHANGELOG.md");

    while !changelog.contains(&expected_header) {
        println!(
            "WARNING: CHANGELOG.md has no entry for version {}.",
            new_version
        );
        println!(
            "Please add a '{}' section to CHANGELOG.md before continuing.",
            expected_header
        );
        println!("Use the commits above as notes for the changelog.\n");

        let user_confirmation_input =
            ask_user_input(&"Have you updated CHANGELOG.md and are ready to continue? [y/N]: ");
        if !(user_confirmation_input.eq_ignore_ascii_case("y")
            || user_confirmation_input.eq_ignore_ascii_case("yes"))
        {
            print_error_and_exit("Aborting release. Update CHANGELOG.md and re-run the script.");
        }

        changelog = fs::read_to_string(changelog_path).expect("Failed to re-read CHANGELOG.md");
    }
    println!("CHANGELOG.md entry verified");

    let updated_cargo_toml = cargo_toml.replace(
        &format!("version = \"{}\"", current_version),
        &format!("version = \"{}\"", new_version),
    );
    fs::write("Cargo.toml", updated_cargo_toml).expect("Failed to write Cargo.toml");
    println!("Updated Cargo.toml");

    println!("\nRunning cargo update...");
    run("cargo", &["update"]);
    println!("cargo update finished");

    println!("\nRunning cargo build --release...");
    run("cargo", &["build", "--release"]);
    println!("cargo build --release finished");

    println!("\nAdding files to git...");
    run("git", &["add", "Cargo.toml", "Cargo.lock", changelog_path]);

    let commit_msg = format!("Release {}", new_version);
    run("git", &["commit", "-m", &commit_msg]);
    println!("Committed changes");

    run("git", &["tag", &new_version]);
    println!("Created tag '{}'", new_version);

    let section = extract_changelog_section(&changelog, &new_version)
        .unwrap_or_else(|| format!("Release {}", new_version));

    println!("\nChangelog section for {}:", new_version);
    println!("{}", section);
    println!();

    let clipcopy_status = Command::new("clipcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(section.as_bytes())?;
            }
            child.wait()
        });

    match clipcopy_status {
        Ok(status) if status.success() => {
            println!("Changelog section copied to clipboard");
        }
        _ => {
            println!("WARNING: Could not copy to clipboard (clipcopy may be missing)");
        }
    }

    println!("\nRelease {} is ready", new_version);
    println!("\nNext steps:");
    println!("  git push origin main");
    println!("  git push origin {}", new_version);
}
