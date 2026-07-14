import re

def main() -> None:
    """Generate release notes for the latest GitHub release."""

    with open("CHANGELOG.md", encoding="utf-8") as file:
        changelog = file.read()

    first_section = re.search(r"^### ", changelog, re.MULTILINE)
    if first_section is None:
        raise RuntimeError("Could not find the first changelog section.")

    next_release = re.search(r"^## ", changelog[first_section.start():], re.MULTILINE)

    if next_release is None:
        raise RuntimeError("Could not find the next release heading.")

    latest_changes = changelog[first_section.start() : first_section.start() + next_release.start()].strip()

    release_notes = f"""A fast code formatter for GDScript in Godot 4.

## Changelog

{latest_changes}

Learn more about the formatter in the [GDScript Formatter documentation](https://www.gdquest.com/library/gdscript_formatter/)
"""

    print(release_notes)

if __name__ == "__main__":
    main()