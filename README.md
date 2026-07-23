# Godot GDScript Formatter

A fast code formatter for Godot's GDScript programming language, built on top of the [Tree Sitter GDScript](https://github.com/PrestonKnopp/tree-sitter-gdscript) parser.

The goal of this project is to provide a fast and reliable GDScript code formatter that's easy to contribute to and to maintain. We also use it to thoroughly test and improve the GDScript parser that powers GDScript support in code editors like Zed, Helix, Neovim and Emacs.

This project aims to follow the official [GDScript style guide](https://docs.godotengine.org/en/stable/tutorials/scripting/gdscript/gdscript_styleguide.html)

**Use a version control system:** Please consider using a version control system like Git to track changes to your code before running the formatter. Even though we use the formatter ourselves at work and it has a large test suite, GDScript is a complex language and there can always be edge cases or less common syntax that may not be handled correctly yet.

## Features

- Format GDScript files perceptually instantly (less than 1ms to a few ms per file on a mid-range laptop)
- Wrap long lines automatically with a configurable maximum line length
- Lint GDScript files for style and convention issues
- Reorder GDScript code to match the official GDScript style guide (variables at the top, then functions, etc.)
- Format code in place (overwrite the file) or print to the standard output
- Check if a file is formatted (for CI/build systems)
- Configure spaces vs tabs, indentation size, blank lines around definitions, and more
- Share formatting settings with your teammates with an `.editorconfig` file
- Exclude code from formatting with `# fmt: off` and `# fmt: on` comments
- Safe mode that prevents overwriting a file if formatting would change the meaning of the code

## Learn how to use

For detailed documentation and guides, check out these pages:

- **[GDScript Formatter docs](https://www.gdquest.com/library/gdscript_formatter/):** Learn how to install the formatter, use it from the command line, and integrate it with your code editor.

- **[Godot 4 addon manual](https://www.gdquest.com/library/gdscript_formatter_godot_addon/):** If you want to use the formatter directly from the Godot editor, this page will tell you how to install, configure, and use the official Godot 4 addon (it works with the latest version of Godot 4).


## Installing and running the formatter

You can find binaries for Windows, macOS, and Linux in the [releases tab](https://github.com/GDQuest/GDScript-formatter/releases) of this repository. Download the binary for your platform, unzip it, rename it to the command name you want (e.g. `gdscript-formatter`) and place it somewhere in your system PATH.

**Alternative sources:**

- **Arch Linux:** Community maintained AUR Package [gdscript-formatter-bin](https://aur.archlinux.org/packages/gdscript-formatter-bin).
- **Windows:** Community maintained Scoop package (`scoop install` [`extras/gdscript-formatter`](https://github.com/ScoopInstaller/Extras/blob/master/bucket/gdscript-formatter.json))
- **GitHub Actions:** Community maintained [setup-gdscript-formatter](https://github.com/Modern-Arts-Research-Stories-Next/setup-gdscript-formatter) action for installing the formatter in GitHub Actions workflows.

To recursively format all GDScript files in the current folder, run:

```bash
gdscript-formatter
```

To format a file, run:

```bash
gdscript-formatter path/to/file.gd
```

To recursively format all GDScript files in another folder, run:

```bash
gdscript-formatter path/to/folder
```

You can also pass multiple files or folders:

```bash
gdscript-formatter path/to/file.gd path/to/folder
```

Use the `--verify-structure` flag to reparse the formatted output and reject it if its structure differs from the input. This is an imperfect check, not a guarantee that formatting is safe or semantically equivalent. It is most useful when formatting many files at once, running the formatter from a script or in continuous integration, or when you do not regularly use version control:

```bash
gdscript-formatter --verify-structure path/to/folder
```

Format with check mode, to use in a build system (exit code 1 if changes needed):

```bash
gdscript-formatter --check path/to/file.gd
```

It will print the files that need to be formatted.

To see other possible options, run `gdscript-formatter --help`.


## Using editorconfig

You can also configure the formatter with an [EditorConfig](https://editorconfig.org/) file at the root of your project. This is a good way to share the same formatting settings with your whole team. The formatter supports the standard keys `indent_style`, `indent_size`, `max_line_length`, `insert_final_newline`, and `trim_trailing_whitespace`, plus custom keys prefixed with `gdscript_formatter_`. See the [GDScript Formatter docs](https://www.gdquest.com/library/gdscript_formatter/) for the complete list. Note that command line flags override `.editorconfig` values.

Use `--quote-style preserve/single/double` to automatically normalize the string quote style. You can also set the style in your `.editorconfig` file using the key `gdscript_formatter_quote_style`. The default value, `preserve`, leaves existing quotes unchanged.

The Godot add-on also reads `gdscript_formatter_format_on_save` from the `.editorconfig` file. This key only affects the add-on and enables or disables format on save for the whole project and overrides each user's add-on setting. For example:

```ini
[*.gd]
gdscript_formatter_format_on_save = true
```

Use this to force everyone in your team to format their GDScript files on save.


## Formatting automatically on commit with pre-commit

You can run the formatter automatically before each commit with a version control hook. If your team uses the pre-commit framework, you can add the formatter hook to your project's `.pre-commit-config.yaml` configuration file:

```yaml
repos:
  - repo: https://github.com/GDQuest/GDScript-formatter
    rev: 0.21.0  # use the tag of the version you want to use
    hooks:
      - id: gdscript-formatter
```

See the [formatter documentation](http://gdquest.com/library/gdscript_formatter#formatting-automatically-on-commit) for some more detailed information on this.


## Keeping a section of code as-is

To keep a section of code exactly as you wrote it, wrap it between `# fmt: off` and `# fmt: on` comments. This is especially useful when you want to keep values aligned or arranged in a specific way when long data structures:

```gdscript
# fmt: off
var matrix = [
	1, 0.5, 0,
	0,   1, 0,
	0,   0, 1,
]
# fmt: on
```

## Linting GDScript files

The formatter also includes a linter that checks for style and convention issues according to the official GDScript style guide.

### Basic linting

To lint a file, run:

```bash
gdscript-formatter lint path/to/file.gd
```

This will output issues in the format:

```
filepath:line:rule:severity: description
```

### Listing available rules

To see all available linting rules:

```bash
gdscript-formatter lint --list-rules
```

### Configuring the linter

#### Disabling specific rules

You can disable specific rules using the `--disable` flag:

```bash
gdscript-formatter lint --disable class-name,signal-name path/to/file.gd
```

#### Setting line length

The linter provides several configurable options:

```bash
gdscript-formatter lint --max-line-length 120 path/to/file.gd
```

#### Pretty printing

By default, the linter outputs one line for each warning/error.

For more human readable output, use the `--pretty` flag:

```bash
gdscript-formatter lint --pretty path/to/file.gd
```

#### Ignoring lines

The linter can be instructed to ignore specific rules for specific lines using special comments.

Ignore a specific rule for the next line:

```gdscript
# gdlint-ignore-next-line private-access
obj._private_method()
```

Ignore a specific rule inline:

```gdscript
obj._private_method() # gdlint-ignore private-access
```

Multiple rules can be ignored with a comma-separated list:

```gdscript
# gdlint-ignore-next-line constant-name,max-line-length
const anotherBadConstName = "this line is also very long but it will be ignored by the comment above this"
```

Ignore ALL rules for the next line:

```gdscript
# gdlint-ignore-next-line
obj._private_method()
```

Ignore ALL rules for the current line inline:

```gdscript
obj._private_method() # gdlint-ignore
```

### List of linter rules

- `function-name` - validates function names (`snake_case`, `_private_snake_case`)
- `class-name` - validates class names (`PascalCase`)
- `signal-name` - validates signal names (`snake_case`)
- `variable-name` - validates class variable names (`snake_case` or `_private_snake_case`)
- `function-argument-name` - validates function argument names (`snake_case` or `_private_snake_case`)
- `loop-variable-name` - validates loop variable names (`snake_case` or `_private_snake_case`)
- `enum-name` - validates enum names (`PascalCase`)
- `enum-member-name` - validates enum element names (`CONSTANT_CASE`)
- `constant-name` - validates constant names (`CONSTANT_CASE`)
- `duplicated-load` - detects copy-pasted load() calls for the same path
- `standalone-expression` - detects standalone expressions that aren't used
- `unnecessary-pass` - detects pass statements when other statements are present
- `unused-argument` - detects unused function arguments
- `comparison-with-itself` - detects redundant comparisons like `x == x`
- `private-access` - detects calls to private methods or variable references (prefixed with `_`)
- `max-line-length` - validates maximum line length
- `no-else-return` - detects unnecessary else after `if`/`elif` blocks that end with `return`

## Using the formatter in code editors

> [!NOTE]
> If you managed to make the formatter work in a code editor that isn't listed here, consider contributing to this section or sharing your findings in [this](https://github.com/GDQuest/GDScript-formatter/issues/26) issue.

In this section, you'll find instructions for setting up the formatter in several code editors.

As a reminder: **use a version control system like Git when turning on format on save**, so you can review the formatter's changes and revert them if needed.

### VSCode

1. Install the [godot-format extension](https://marketplace.visualstudio.com/items?itemName=DoHe.godot-format) in VSCode. Press `Ctrl+P` and run:

```
ext install DoHe.godot-format
```

2. This extension ships with the formatter binary pre-installed, so you don't need to download the formatter separately.

Once installed, visit the [extension page](https://marketplace.visualstudio.com/items?itemName=DoHe.godot-format) to see the available settings and set your preferences.

### Zed

1. Install the formatter (see instructions above).

2. Install [zed-gdscript](https://github.com/GDQuest/zed-gdscript) extension. This is needed to ensure that the formatter will only format GDScript files.

3. After installing the extension, add the following JSON configuration to your `settings.json` file:

```json
{
  "languages": {
    "GDScript": {
      "formatter": {
        "external": {
          "command": "gdscript-formatter",
          "arguments": []
        }
      }
    }
  }
}
```

4. If you renamed the binary to something else during installation, adjust the `command` name accordingly. It can also be a full path to the binary.

5. `arguments` is a comma separated list of flags. For example, to use spaces instead of tabs, replace `[]` with `["--use-spaces"]`. I recommend running the `--reorder-code` feature manually rather than on every save, as it makes big changes to your files.

Once this is done, you can start using the formatter. By default Zed will run the formatter every time you save a file. If don't want this to happen, set the `format_on_save` setting in `settings.json` to `false`. To format manually, execute the `editor: format` command in Zed.

As a reminder: **use a version control system like Git when leaving format on save enabled!**

### Helix

1. Follow the [instructions](https://www.gdquest.com/library/gdscript_formatter/) carefully on installing the formatter and make sure it's in your **PATH**.

2. Go to Helix config directory to edit your languages configuration file. Since I use helix as my terminal editor and I'm on macOS, I'll open it up with the **hx** command:

```bash
cd ~/.config/helix && hx languages.toml
```

3.  Add this line inside your **[[Language]]** block assigned to gdscript:

```toml
formatter = { command = "gdscript-formatter" }
```

Keep in mind, using gdscript with Helix [requires more configuration](https://github.com/helix-editor/helix/blob/master/languages.toml) than this, including [changing a few options inside Godot editor](https://docs.godotengine.org/en/stable/tutorials/editor/external_editor.html) and possibly making a script for activating Helix in your terminal of choice.

4. Auto-format on save can be enabled by adding this line to your gdscript language options, as shown in the linked example at Helix repository:

```toml
auto-format = true
```

### JetBrains Rider

1. First, install the formatter on your computer.
2. Open Rider and go to your IDE settings. You can find them under `Tools > File Watchers`.
3. Click the `+` button to add a new file watcher and pick `<custom>` from the dropdown list.
4. Now fill in these fields:
   - **Name**: GDScript Formatter
   - **File Type**: GDScript
   - **Scope**: Current File
   - **Program**: `gdscript-formatter` (or write the full path to the binary if it's not in your **PATH**)
   - **Arguments**: `$FilePath$`
   - **Output Paths to refresh**: `$FilePath$`
   - **Working Directory**: `$ModuleFilePath$`
   - You can optionally check any of the checkboxes for auto-save and triggering the watcher when files change outside the editor.
   - Keep the box for `Create output file from stdout` unchecked.


If you lose work because of the formatter, you can usually get it back with a simple "undo" (Cmd/Ctrl + Z). This will show you the "undo reload from disk" popup. You can also check the local history by right-clicking on the file in the project explorer and selecting `Local History > Show History`.

## Contributing

Contributions are welcome! I've compiled some guides and guidelines below to help you get started with contributing to the GDScript formatter. If you need more information or want to discuss ideas for the formatter, please get in touch on the [GDQuest Discord](https://discord.gg/87NNb3Z).

**Please report any issues you find with code snippets!** GDScript has grown into a complex language with many different syntax patterns. While the formatter covers many common cases, there can always be edge cases or less common syntax that may not be handled correctly yet. You can find known issues in the [GitHub issues section](https://github.com/GDQuest/GDScript-formatter/issues).

### Building the formatter locally for development

To build the formatter locally for testing, you need the Rust language compiler and the Rust language build system `cargo`. Then you can run:

```bash
cargo build
```

It'll download all the dependencies, compile them, and build a binary in a `target/debug/` folder. You can then run the built program with `cargo run -- [args]`.

### How the formatter works

The formatter runs in three broad steps:

1. Parsing: the [tree-sitter](https://tree-sitter.github.io/tree-sitter/) GDScript parser turns the source code into a syntax tree. Tree-sitter is a parser generator used by many code editors for syntax highlighting and code navigation (Zed, Neovim, Emacs, Helix...).
2. Formatting: the formatter walks the syntax tree and builds an intermediate representation of the formatted code (see `src/formatter.rs`).
3. Rendering: the renderer turns the intermediate representation into the final string, wrapping lines that go over the maximum line length (see `src/renderer.rs`).

### Adding new formatting rules

To add new formatting rules to the GDScript formatter, you can follow these steps:

1. Add test cases with real-world GDScript code: To add a test case, create input/expected file pairs in `tests/input/` and `tests/expected/` respectively. For example, if you want to test a new rule for function definitions, create `tests/input/function_definition.gd` and `tests/expected/function_definition.gd`:
   - The input file contains the GDScript code before running the formatter
   - The expected file contains the GDScript code after applying the new formatting rules
2. Run tests: Use `cargo test` to run the formatter on every input/expected file pair in the `tests/` directory. This will check if the formatter produces the expected output for each of them
3. Implement the rule: Most formatting rules live in `src/formatter.rs`, which builds the intermediate representation from the syntax tree. Line wrapping and output happen in `src/renderer.rs`. The file `src/node_kind.rs` lists the GDScript AST node kinds you can match against.

### Development resources

- **[Tree-sitter documentation](https://tree-sitter.github.io/tree-sitter/)**: Reference for the parser technology the formatter is built on. Understanding how tree-sitter represents code as a syntax tree helps a lot when working on formatting rules
- **[GDScript Style Guide](https://docs.godotengine.org/en/stable/tutorials/scripting/gdscript/gdscript_styleguide.html)**: Official Godot style guidelines

## Project structure

Here are the most important directories and files in the project:

- `src/`: Contains the code of the formatter and the linter. Quick run through the files: `formatter.rs` (formatting rules), `renderer.rs` (line wrapping and output), `reorder.rs` (code reordering), `safe_mode.rs` (the safe mode check), `editorconfig.rs` (EditorConfig support), and the `linter/` directory (linter rules).
- `tests/`: Contains test files for the formatter. It has input files with unformatted GDScript code and expected output files that the formatter should produce when run on the input files, plus dedicated tests for the linter and the reorder feature.
- `benchmarks/`: Contains GDScript files used to measure the formatter's performance.
- `addons/`: Contains the source of the Godot editor add-on.
- `docs/`: Contains the source of the user documentation and coding guidelines for contributors.

### Development workflow

To test formatting on a simple code snippet, you can use `echo` or `cat` to pass GDScript code to the formatter. This is useful for quick tests since the output is directly printed to the console.

```bash
echo 'var x=1+2' | cargo run --quiet
```

To run the formatter on a file without overwriting it, use the `--stdout` flag:

```bash
cargo run --quiet -- --stdout test.gd
```

### Running tests

To run the formatter's test suite, use this command:

```bash
cargo test
```

### Benchmarking and profiling

Run the timing benchmark to compare total formatter runtime between revisions:

```bash
cargo run --bin benchmark --release
```

The output result is the median time of multiple batches of formatter runs over the same files.

To profile the run and see which functions use the most CPU time, I use the sampling profiler [samply](https://github.com/mstange/samply). Install it like this, then I've prepared a script to run the formatter while running the profiler:

```bash
cargo install --locked samply
./benchmarks/profile.sh
```

Samply allows you to view the profiling results in Firefox's profiler UI.

The script runs 5000 iterations by default, but you can pass a different count as an argument, for example `./benchmarks/profile.sh 10000`.

### Debugging and visualizing the tree structure

You can use the tree-sitter command line tool to parse GDScript files and visualize the syntax tree. This shows you the structure that the tree-sitter parser generates, which you can then use to write formatting rules:

```bash
tree-sitter parse --scope source.gdscript test.gd
```

This requires setting up tree-sitter on your computer and having the GDScript parser configured.

When you're getting started with contributing to the formatter, I recommend beginning with this command. For example, if you want to change how function definitions are formatted, you'd first use it to see how the parser represents functions in the tree. Then you can find where the formatter processes those nodes in `src/formatter.rs`.

## License

[MIT](https://github.com/GDQuest/GDScript-formatter/blob/main/LICENSE)

## Motivation

The Godot team has wanted an official GDScript formatter since the early days, but it has always been part of the engine's development backlog. It's a tool Godot users would use daily, so in 2022, we set out to sponsor the development of an [official GDScript formatter](https://github.com/godotengine/godot/pull/76211) built into Godot 4.

We put a lot of work into this project at GDQuest. Then, following the suggestion to break up the work into smaller contributions, a dedicated contributor, Scony, took over the project and tried breaking down the implementation [into small chunks](https://github.com/godotengine/godot/pull/97383) to make it much easier to review and merge. However, there isn't an active maintainer to review and merge the work, and the project has been stuck for a while now.

Scony has been maintaining a solid set of community tools for GDScript, including a code formatter written in Python: [Godot GDScript Toolkit](https://github.com/Scony/godot-gdscript-toolkit). It's a great project that many Godot developers have used. So, why start another one?

The main reason to try this project is that Scony's formatter has grown quite complex over the years, and it has limitations for us at GDQuest that make it not work for our projects. Some of these limitations are also not easy to fix. For instance, the fact it's implemented in Python with the LARK library for parsing limits the performance.

Since Scony made his great formatter, new technologies have come up that could make it much easier to build and maintain one, like Tree-sitter. This project started as a suggestion from one of Godot's core contributors to test these new technologies for GDScript formatting. The first version of the formatter was built on [Topiary](https://topiary.tweag.io/), a formatting engine based on Tree-sitter, which let us ship a working tool very quickly. Users then asked for features that Topiary couldn't support well, like automatic line wrapping, so we rewrote the formatter with our own formatting and rendering code on top of the Tree-sitter parser. This also made the formatter faster.

<details>
<summary>What is Tree-sitter?</summary>
Tree-sitter is a powerful parser generator that makes it easy to create programming language parsers that run natively (it generates C code). It's used in modern code editors for syntax highlighting, code navigation, outline, folding, and more (Zed, Neovim, Emacs, Helix, Lapce...).
</details>
