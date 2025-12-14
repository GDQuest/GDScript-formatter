/// Release automation script for GDScript Formatter
///
/// This script automates the release process by:
/// 1. Prompting for version bump type (major, minor, or patch)
/// 2. Verifying the git tag doesn't already exist
/// 3. Updating the version in Cargo.toml
/// 4. Running `cargo update` to update Cargo.lock
/// 5. Running `cargo build --release` to verify the build works
/// 6. Committing the changes with a standardized message
/// 7. Creating a git tag with the new version
/// 8. Generating a changelog and copying it to clipboard
///
/// Usage:
///   cargo run --bin make_release
///
/// After the script completes, you'll need to manually push:
///   git push origin main
///   git push origin <version>
use std::fs;
use std::io::{self, Write};
use std::process::{Command, exit};

fn main() {
    println!("=== GDScript Formatter Release Script ===\n");

    let cargo_toml = fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");

    let current_version = cargo_toml
        .lines()
        .find(|line| line.starts_with("version = "))
        .and_then(|line| line.split('"').nth(1))
        .expect("Failed to find version in Cargo.toml");

    println!("Current version: {}", current_version);

    let parts: Vec<&str> = current_version.split('.').collect();
    if parts.len() != 3 {
        eprintln!("Invalid version format");
        exit(1);
    }
    let (major, minor, patch) = (
        parts[0].parse::<u32>().unwrap(),
        parts[1].parse::<u32>().unwrap(),
        parts[2].parse::<u32>().unwrap(),
    );

    print!("\nVersion bump type? [major/minor/patch]: ");
    io::stdout().flush().unwrap();
    let mut bump_type = String::new();
    io::stdin().read_line(&mut bump_type).unwrap();
    let bump_type = bump_type.trim().to_lowercase();

    let new_version = match bump_type.as_str() {
        "major" => format!("{}.0.0", major + 1),
        "minor" => format!("{}.{}.0", major, minor + 1),
        "patch" => format!("{}.{}.{}", major, minor, patch + 1),
        _ => {
            eprintln!("Invalid bump type. Use major, minor, or patch.");
            exit(1);
        }
    };

    println!("\nNew version will be: {}", new_version);

    let tag_exists = Command::new("git")
        .args(["rev-parse", &new_version])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if tag_exists {
        eprintln!("Error: Git tag '{}' already exists!", new_version);
        exit(1);
    }

    println!("✓ Git tag does not exist");

    let updated_cargo_toml = cargo_toml.replace(
        &format!("version = \"{}\"", current_version),
        &format!("version = \"{}\"", new_version),
    );
    fs::write("Cargo.toml", updated_cargo_toml).expect("Failed to write Cargo.toml");
    println!("✓ Updated Cargo.toml");

    println!("\nRunning cargo update...");
    let update_status = Command::new("cargo")
        .arg("update")
        .status()
        .expect("Failed to run cargo update");

    if !update_status.success() {
        eprintln!("cargo update failed");
        exit(1);
    }
    println!("✓ cargo update successful");

    println!("\nRunning cargo build...");
    let build_status = Command::new("cargo")
        .args(["build", "--release"])
        .status()
        .expect("Failed to run cargo build");

    if !build_status.success() {
        eprintln!("cargo build failed");
        exit(1);
    }
    println!("✓ cargo build successful");

    println!("\nAdding files to git...");
    let add_status = Command::new("git")
        .args(["add", "Cargo.toml", "Cargo.lock"])
        .status()
        .expect("Failed to run git add");

    if !add_status.success() {
        eprintln!("git add failed");
        exit(1);
    }

    let commit_msg = format!("Update to version {}", new_version);
    let commit_status = Command::new("git")
        .args(["commit", "-m", &commit_msg])
        .status()
        .expect("Failed to run git commit");

    if !commit_status.success() {
        eprintln!("git commit failed");
        exit(1);
    }
    println!("✓ Committed changes");

    let tag_status = Command::new("git")
        .args(["tag", &new_version])
        .status()
        .expect("Failed to run git tag");

    if !tag_status.success() {
        eprintln!("git tag failed");
        exit(1);
    }
    println!("✓ Created tag '{}'", new_version);

    println!("\nGenerating changelog...");
    let shortlog_output = Command::new("git")
        .args(["shortlog", &format!("{}..HEAD", current_version)])
        .output()
        .expect("Failed to run git shortlog");

    let shortlog = String::from_utf8_lossy(&shortlog_output.stdout);
    println!("\n--- Changelog ---");
    println!("{}", shortlog);
    println!("--- End Changelog ---\n");

    let clipcopy_status = Command::new("clipcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(shortlog.as_bytes())?;
            }
            child.wait()
        });

    match clipcopy_status {
        Ok(status) if status.success() => {
            println!("✓ Changelog copied to clipboard");
        }
        _ => {
            println!("⚠ Failed to copy to clipboard (clipcopy might not be available)");
        }
    }

    println!("\n=== Release {} ready! ===", new_version);
    println!("\nNext steps:");
    println!("  git push origin main");
    println!("  git push origin {}", new_version);
}
