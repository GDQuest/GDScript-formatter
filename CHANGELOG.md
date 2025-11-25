# Changelog

This file documents the changes made to the formatter with each release. This project uses [semantic versioning](https://semver.org/spec/v2.0.0.html).

## Release 0.18.1 (2025-11-25)

### Fixed

- `if/elif/else` statements on single lines wrapping incorrectly (#175)
- Godot addon: support for non-English characters

## Release 0.18.0 (2025-11-18)

### Added

- Godot addon: allow using both reorder code and safe mode together

### Fixed

- Safe mode: search for `extends_statement` node instead of accessing it by index (#174)
- Multiline lambdas in ternaries having too much indentation and getting a parse error
- Remove version number from Godot addon zip in releases (it was breaking download links)

## Release 0.17.0 (2025-11-10)

### Added

- Version number now included in release artifacts

### Changed

- CI: updated the release workflow to ensure the build completes before creating a release

## Release 0.16.1 (2025-11-10)

### Fixed

- Godot addon: Use standard output mode to avoid overwriting the file on disk

### Changed

- Moved make_release and benchmark scripts to src/bin folder to avoid build errors

## Release 0.16.0 (2025-11-09)

### Added

- Safe mode support for reorder code to verify syntax tree integrity after reordering

### Changed

- Godot addon: safe mode is now enabled by default
- Improved handling of lists with leading commas and inline comments

### Fixed

- Reorder code: fixed inline comments wrapping to separate lines
- Added test case for comma after lambda function argument

## Release 0.15.0 (2025-11-09)

### Added

- Icon for the program and the Godot addon

### Changed

- Updated `topiary-core` to v0.7.0
- Updated GDScript parser; added test for comments inside dictionaries

### Fixed

- Safe mode: fixed panic when annotation doesn't have a name
- Fixed reorder mode dropping class-level `@abstract` annotation
- Fixed reorder mode dropping trailing comments and reordering comments in exported declarations
- Godot addon: fixed shortcut not working
- Added test case for indented comment at the end of a function body

### Meta

- Applied clippy fixes

## Release 0.14.0 (2025-10-10)

### Added

- Linting support to the Godot addon
- Instructions on how to use the formatter in VSCode to the README
- CI action to run tests on push
- CI job to package the Godot add-on to zip it automatically on releases
- Remove trailing whitespace when formatting (outside of strings)

### Changed

- Removed alphabetic reordering when using the --reorder-code option

### Fixed

- Comment indentation test case

## Release 0.13.1 (2025-10-07)

## Fixed

- Fixed an issue preventing the program from building and releasing the latest version.

## Release 0.13.0 (2025-10-06)

### Added

- `max_term_width` configuration option now set to 120 characters by default

### Changed

- Improved `--help` and `-h` output for better clarity
- Updated the tree-sitter GDScript parser to the latest version
- Function annotations now preserve input formatting and allow inline annotations with function definitions

### Fixed

- Safe mode no longer moves inline annotations in functions
- CI: Removed rust cache as it cannot work for the current build process
- Fixed issue with comments dedented from a function body causing next lines to be dedented too
- Fixed reorder mode dropping comments in one edge case (multiple commented lines at different indent levels at the end of a function)

## Release 0.12.0 (2025-10-03)

### Added

- New `lint` command with 17 code style and quality rules and warnings to help catch common issues
- File name is now printed when formatting a single file

### Changed

- Updated documentation to consistently use `gdscript-formatter` instead of `gdscript-format`

### Fixed

- Annotations not being parsed correctly in safe mode

## Release 0.11.1 (2025-10-01)

This release fixes issues with class docstring formatting and comments at the end of functions most notably.

### Added

- Idempotence checks to all tests to ensure formatted code remains unchanged when formatted again
- Test cases for trailing comments and await expressions

### Fixed

- Docstrings being incorrectly attached to classes instead of following statements during code reordering
- Comments at the very end of functions being incorrectly moved to a new line
- Await expressions being incorrectly formatted when used with `not`

## Release 0.11.0 (2025-09-29)

This release improves the formatter's performance by up to 10% compared to previous release, fixes several issues with safe mode, and refines the handling of docstrings and region.

### Added

- Formatting support for class documentation comments
- Improved spacing logic for inline comments in cases where we need to apply two lines of spacing

### Changed

- Improved the performance of `class_name` and `extends` queries
- Updated safe mode help message to warn about rare cases where it may not catch all formatting issues

### Fixed

- Safe mode incorrectly parsing `extends` statements as children of `class_name` statements
- `#endregion` comments immediately following `#region` comments being collapsed
- Missing empty lines between class documentation comments and following statements
- Multiple empty lines after `extends` statements (now we ensure there is only one empty line)
- Godot add-on: Download URL for Windows users failing due to incorrect file extension
- Failing reorder test cases were cleaned up and corrected

## Release 0.10.1 (2025-09-28)

### Changed

- Restored and added code reordering test cases

### Fixed

- Edge cases with class name and extend wrapping not being enforced on two lines
- Reorder mode dropping region comments and RPC annotations
- Some multiline strings being parsed incorrectly
- Class docstring first line being moved incorrectly during reordering
- Typed dictionaries function return types not formatting correctly
- Region comments not attaching to the correct code block after reordering

## Release 0.10.0 (2025-09-27)

This release introduces a Godot addon to integrate the formatter with the Godot editor and fixes several formatting edge cases.

### Added

- Godot addon in which you can:
  - Install and uninstall the formatter directly from within the Godot editor
  - Access issues and documentation
  - Use the formatter, reorder code, and format on save
  - Change formatter settings
- Reference to the AUR package in the README

### Changed

- Changed formatting for `class_name` declarations with `extends` to wrap on two lines (following the official style guide)

### Fixed

- Newlines being incorrectly removed in multiline ternary expressions
- Formatting issues with `extends` when the class name is under 3 characters long
- Missing line returns between multiple match patterns on a single line

## Release 0.9.1 (2025-09-25)

This is a minor release that adds Nix support and fixes several edge cases with output formatting and comma handling. It also prepares for support for the formatter from within Godot.

### Added

- Nix flake support to use the formatter on NixOS

### Changed

- Release artifacts now only include zip files instead of both zip and tar.gz (this is needed to support auto-install in Godot)
- Removed cleanup for lines containing only whitespace

### Fixed

- Progress messages appearing in output when using the `--stdout` option
- Dangling commas sometimes being incorrectly moved inside strings
- Various edge cases in post-processing formatting

## Release 0.9.0 (2025-09-25)

This release focuses on performance improvements and adds support for formatting multiple files at once.

### Added

- Support for formatting multiple files at once
- Multi-threading when formatting multiple files for better performance

### Changed

- Improved performance on long GDScript files by 5 to 10%
- Don't parse code multiple times when using `--safe` flag
- Reuse parser instances and trees to reduce memory allocations
- Updated dependencies to latest versions
- Updated Zed editor configuration instructions

### Fixed

- Fixed commas ending up dangling on separate lines in some cases (after lambdas in function calls, arrays, and dictionaries)
- Don't modify original syntax tree for safety checks

## Release 0.8.1 (2025-09-23)

### Changed

- Shifted to a fork of the GDScript parser to allow addressing upstream issues sooner

### Fixed

- Incorrect formatting of functions defined inline with class definitions
- Space added in function parameter type inference
- Support for annotations in match patterns
- Support for conditional expressions in match statements
- Comments sometimes being aligned to the previous line indentation level instead of the current one in functions and classes
- Definitions inline with a class definition not being wrapped to a new line

## Release 0.8.0 (2025-09-22)

This release adds a safe mode to help make the formatter more resilient, and adds configuration instructions to integrate it into Zed, Helix, and JetBrains Rider.

### Added

- Syntax tree verification option (`--safe`) to catch formatting issues early
- Instructions to integrate the formatter into JetBrains Rider
- Two blank lines between functions/inner classes and following variable, signal, enum, or constant declarations

### Fixed

- Space after bitwise NOT being incorrectly added
- Comma placement after multiline arrays
- Trailing comma being added to multiline preload calls (GDScript does not support trailing commas in this case)

## Release 0.7.1 (2025-09-21)

This release brings two hotfixes and adds configuration instructions for two editors.

### Added

- Instructions to integrate the formatter into the Zed and Helix editors

### Fixed

- Regions being erased during formatting
- Inline comments in function calls being misplaced

## Release 0.7.0 (2025-09-19)

This release improves formatting consistency for dictionaries, arrays, and function parameters, and fixes several edge cases related to variable declarations and class definitions.

### Added

- Space after comma between types in typed dictionary type hints
- Space after the opening brace and before closing braces of single line dictionaries
- Trailing commas in arrays, dictionaries, enums, and function parameters
- New line before enums closing brace
- Removed trailing comma in singleline arrays/dictionaries/enums/functions

### Fixed

- Variable declaration after function getting placed inside function (or class)
- Incorrect formatting when a declaration immediately follows an extends statement in an inner class

## Release 0.6.0 (2025-09-19)

This release improves formatting consistency and fixes several edge cases related to spacing, comments, and function/class definitions.

### Added

- Space after commas in setter and getter declarations
- Two blank lines before annotated functions

### Changed

- Improved the detection and removal of dangling semicolons
- Removed unnecessary topiary rule for `@tool` annotations

### Fixed

- Functions with a single statement on a single line (e.g., `func a(): pass`) being incorrectly merged with the following function
- Class definitions placed next to one another losing line breaks between them
- Inline comments after annotations being misplaced
- Comments in arrays and dictionaries being incorrectly formatted in some cases
- Comments in enums being misaligned in some cases

## Release 0.5.1 (2025-09-18)

This release fixes critical bugs that could cause data loss during formatting.

### Added

- Formatting support for pattern guards (syntax: `a when b:`)
- Test cases for string literals to prevent regressions

### Changed

- Added warning in README about using version control systems when formatting code to prevent data loss

### Fixed

- StringName strings (`&"TextHere"`) being erased during formatting
- NodePath strings (`^"Path/To/Node"`) being erased during formatting

## Release 0.5.0 (2025-09-18)

This release greatly improves the performance of the formatter, which makes it feel even snappier than before. The time to format is divided by up to 2.

### Added

- Support for multiline function calls with correct indentation
- Option to reorder GDScript code according to the official style guide
- Benchmark script to test the formatter's performance on small and large files (run `cargo run --bin benchmark --release`)

### Changed

- Updated GDScript tree-sitter parser and tree-sitter library to the latest version, bringing a big performance improvement (up to 30%)
- Optimized release builds with lto compile flags (this brings a 10-20% speed improvement)
- Improved vertical spacing between class-level declarations to add two lines even if there are docstrings
- Improved module documentation and docstrings
- Vertical spacing logic to account for multi-line comments/docstrings before definitions
- Refactored formatter to use more idiomatic Rust (the formatter is now a struct and multiline module comments are docstrings)
- `gdscript-formatter` is now the default binary for `cargo run`

### Fixed

- Loss of node names/paths in `%` and `$ get_node` syntaxes
- Leading space before `not` being lost during formatting in the expression `not in`
- Line continuation markers being lost upon formatting
- Incorrect GitHub URLs in README (#15)

## Release 0.4.0 (2025-09-10)

### Fixed

- Trailing comments at the end of functions were being wrapped on a new line. They're now preserved at the end of the function line.

### Changed

- Updated to latest version of the GDScript parser with adapted queries for new body node in setters and getters
- Added test case for trailing comments at the end of functions to ensure correct formatting

## Release 0.3.0 (2025-09-04)

### Added

- Print the help message if there are no arguments or piped input

### Fixed

- Semicolons: wrap statements on multiple lines when needed, preserve indentation in code blocks
- Inline comments after colons wrapping on another line

### Changed

- Make tests run much 3 to 4x faster and greatly improve output diff
- Use cargo configuration to strip debug symbols from release binaries

## Release 0.2.0 (2025-08-23)

### Added

- Support for multi-line wrapping of function parameters with extra indentation
- Spacing around the "as" keyword

### Changed

- Formatter now overwrites formatted files by default instead of outputting to stdout
- Added option to output to stdout when needed
- Version number is now read directly from Cargo.toml at build time

## Release 0.1.0 (2025-08-21)

This is the initial release of the GDScript formatter.

### Added

- Support for many GDScript formatting rules:
  - Consistent spacing between operators, keywords, and after commas in most cases
  - Single and multi-line formatting for arrays and dictionaries
  - Consistent indentation for blocks, function definitions, and control structures
  - Enforces blank lines between functions and classes
- Configuration option for indentation (spaces or tabs)
- Test suite with input/expected file pairs (run with `cargo test`)
- Cross-platform support (Linux, macOS, Windows) and automated builds with GitHub Actions
