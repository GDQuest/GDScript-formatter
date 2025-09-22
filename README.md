# Godot GDScript Formatter

A fast code formatter for Godot's GDScript programming language built with [Tree Sitter GDScript](https://github.com/PrestonKnopp/tree-sitter-gdscript) and [Topiary](https://topiary.tweag.io/).

The goal of this project is to provide a simple and really fast GDScript code formatter that's easy to contribute to, and easy to maintain. It also benefits GDScript support in code editors like Zed, Neovim, and Emacs as we use the project to improve the Tree Sitter GDScript parser.

**Use a version control system:** Please consider using a version control system like Git to track changes to your code before running the formatter. Even though we already use the formatter ourselves at work, GDScript is a complex language and a formatter needs testing on all possible syntax combinations in the language to ensure the output is rock solid. There can always be edge cases or less common syntax that may not be handled correctly yet.

## Features

- Format GDScript files nearly instantly (less than 100ms for a 1000-line file on a mid-range laptop, less than 30ms for most files)
- Reorder GDScript code to match the official GDScript style guide (variables at the top, then functions, etc.)
- Format code in place (overwrite the file) or print to the standard output
- Check if a file is formatted (for CI/build systems)
- Configure spaces vs tabs and indentation size

## Installing and running the formatter

You can find binaries for Windows, macOS, and Linux in the [releases tab](https://github.com/GDQuest/godot-gdscript-formatter-tree-sitter/releases) of this repository. Download the binary for your platform, unzip it, rename it to the command name you want (e.g. `gdscript-format`) and place it somewhere in your system PATH.

To format a file, run:

```bash
gdscript-format path/to/file.gd
```

Format with check mode, to use in a build system (exit code 1 if changes needed):

```bash
gdscript-format --check path/to/file.gd
```

To see other possible options, run `gdscript-format` without any arguments.

## Using the formatter in code editors

> [!NOTE]
> If you managed to make the formatter work in a code editor that isn't listed here, consider contributing to this section or sharing your findings in [this](https://github.com/GDQuest/GDScript-formatter/issues/26) issue.

### Zed

First, install the [zed-gdscript](https://github.com/GDQuest/zed-gdscript) extension. This is needed to ensure that the formatter will only format GDScript files. Once installed, add the following JSON configuration to your `settings.json` file:

```json
{
  "languages": {
    "GDScript": {
      "formatter": {
        "external": {
          "command": "gdscript-format"
        }
      }
    }
  }
}
```

If you renamed the binary to something else, adjust the `command` name accordingly. Once this is done, Zed will run the formatter every time you save a GDScript file. If this doesn't happen, ensure that the `format_on_save` setting in `settings.json` is set to `true` (this is the default). You can also format manually by executing `editor: format` command in Zed.

### Helix

1. Follow the [instructions](https://www.gdquest.com/library/gdscript_formatter/) carefully on installing the formatter and make sure it's in your **PATH**.

2. Go to Helix config directory to edit your languages configuration file. Since I use helix as my terminal editor and I'm on macOS, I'll open it up with the **hx** command:

```bash
cd ~/.config/helix && hx languages.toml
```

3.  Add this line inside your **[[Language]]** block assigned to gdscript:

```toml
formatter = { command = "gdscript-formatter", args = ["--reorder-code"] }
```

Keep in mind, using gdscript with Helix [requires more configuration](https://github.com/helix-editor/helix/blob/master/languages.toml) than this, including [changing a few options inside Godot editor](https://docs.godotengine.org/en/stable/tutorials/editor/external_editor.html) and possibly making a script for activating Helix in your terminal of choice.

4. Auto-format on save can be enabled by adding this line to your gdscript language options, as shown in the linked example at Helix repository:

```toml
auto-format = true
```

As a reminder: **don't leave this on when working on an important project without using a VCS!**.

### JetBrains Rider

1. First, install the formatter on your computer.
2. Open Rider and go to your IDE settings. You can find them under `Tools > File Watchers`.
3. Click the `+` button to add a new file watcher and pick `<custom>` from the dropdown list.
4. Now fill in these fields:
   - **Name**: GDScript Formatter
   - **File Type**: GDScript
   - **Scope**: Current File
   - **Program**: `gdscript-format` (or write the full path to the binary if it's not in your **PATH**)
   - **Arguments**: `$FilePath$` or `$FilePath$ --reorder-code`
   - **Output Paths to refresh**: `$FilePath$`
   - **Working Directory**: `$ModuleFilePath$`
   - You can optionally check any of the checkboxes for auto-save and triggering the watcher when files change outside the editor.
   - Keep the box for `Create output file from stdout` unchecked.

As a reminder: **don't turn this on when working on an important project without using a VCS like Git!**.

If you lose work because of the formatter, you can usually get it back with a simple "undo" (Cmd/Ctrl + Z). This will show you the "undo reload from disk" popup. You can also check the local history by right-clicking on the file in the project explorer and selecting `Local History > Show History`.

## Status

09/18/2025 - The formatter now has many formatting rules implemented and is ready to test. It includes:

- **Spaces**: leaving one space consistently between many operators, most keywords, or after commas in function calls, arrays, and dictionaries
- **Multi-line structures**: simple arrays and dictionaries can be wrapped on one or multiple lines with indentation
- **Indentation**: consistent indentation for blocks, function definitions, and control structures with configurable indent types (tabs or spaces)
- **Vertical spacing**: proper blank lines between functions, classes, and other major code structures

And more!

**Please report any issues you find with code snippets!** GDScript has grown into a complex language with many different syntax patterns. While the formatter covers many common cases, there can always be edge cases or less common syntax that may not be handled correctly yet. You can find known issues in the [GitHub issues section](https://github.com/GDQuest/godot-gdscript-formatter-tree-sitter/issues).

### Formatting on single or multiple lines

The formatter's technology doesn't handle maximum line length automatically. Instead, for wrapping code on a single or multiple lines, it uses cues from you, the developer. For example, if you write an array on a single line, it will remain on a single line. This input:

```gdscript
var numbers: Array[int] = [1,2,3,4,5]
```

Will be formatted like this:

```gdscript
var numbers: Array[int] = [1, 2, 3, 4, 5]
```

If you insert a line return, the array will wrap on multiple lines instead. This input:

```gdscript
var dialogue_items: Array[String] = ["I'm learning about Arrays...",
	"...and it is a little bit complicated.", "Let's see if I got it right: an array is a list of values!", "Did I get it right? Did I?", "Hehe! Bye bye~!"]
```

Will be formatted like this:

```gdscript
var dialogue_items: Array[String] = [
	"I'm learning about Arrays...",
	"...and it is a little bit complicated.",
	"Let's see if I got it right: an array is a list of values!",
	"Did I get it right? Did I?",
	"Hehe! Bye bye~!"
]
```

You can insert the line returns anywhere in the array, and the formatter will keep it on multiple lines. The same applies to other structures.

## Contributing

Contributions are welcome! I've compiled some guides and guidelines below to help you get started with contributing to the GDScript formatter. If you need more information or want to discuss ideas for the formatter, please get in touch on the [GDQuest Discord](https://discord.gg/87NNb3Z).

### Building the formatter locally for development

To build the formatter locally for testing, you need the Rust language compiler and the Rust language build system `cargo`. Then you can run:

```bash
cargo build
```

It'll download all the dependencies, compile them, and build a binary in a `target/debug/` folder. You can then run the built program with `cargo run -- [args]`.

### Adding new formatting rules

To add new formatting rules to the GDScript formatter, you can follow these steps:

1. **Add test cases with real-world GDScript code**: To add a test case, create input/expected file pairs in `tests/input/` and `tests/expected/` respectively. For example, if you want to test a new rule for function definitions, create `tests/input/function_definition.gd` and `tests/expected/function_definition.gd`:
   - The input file contains the GDScript code before running the formatter
   - The expected file contains the GDScript code after applying the new formatting rules
2. **Run tests**: Use `cargo test` to run the formatter on every input/expected file pair in the `tests/` directory. This will check if the formatter produces the expected output for each of them
3. **Update queries**: Modify `queries/gdscript.scm` with the formatting rules. This is the file that defines how the formatter should format GDScript code. You can use the existing rules as a reference for writing new ones (and the topiary documentation links below for more details)

### Development resources

- **[Tree-sitter Query Syntax](https://tree-sitter.github.io/tree-sitter/using-parsers/queries/1-syntax.html)**: Reference for writing tree-sitter queries - it's essential to understand how to write queries for formatting
- **[Topiary Documentation](https://topiary.tweag.io/book/)**: Complete guide to query syntax and formatting
- **[GDScript Style Guide](https://docs.godotengine.org/en/stable/tutorials/scripting/gdscript/gdscript_styleguide.html)**: Official Godot style guidelines

**Important**: we will likely not be able to implement all the guidelines from the official style guide with this formatter. What we gain in ease of implementation and maintenance, we lose in flexibility and advanced patterns.

## Project structure

Here are the most important directories and files in the project:

- `src/`: Contains the Rust code to compile and run the formatter using the CLI. It's currently a simple wrapper around the Topiary command line program, but later it could use Topiary as a library instead to pack everything into a simple binary.
- `tests/`: Contains test files for the formatter. It has input files with unformatted GDScript code and expected output files that the formatter should produce when run on the input files.
- `queries/`: Contains the Topiary formatting rules for GDScript. The `gdscript.scm` file is where you define how GDScript code should be formatted based on Tree Sitter queries and Topiary features to mark nodes/patterns for formatting.
- `config/`: Contains configuration files for Topiary - basically a small file that tells Topiary how to run the formatter for GDScript.
- `docs/`: This folder will compile images and recaps or cheat sheets with some tricks to help when working with Tree Sitter queries and Topiary.

### Development workflow

To test formatting on a simple code snippet, you can use `echo` or `cat` to pass GDScript code into the Topiary formatter. This is useful for quick tests since the output is directly printed to the console.

```bash
echo 'var x=1+2' | TOPIARY_LANGUAGE_DIR=queries topiary format --language gdscript --configuration config/languages.ncl
```

Running the formatter on a file is also supported, but note that it overwrites the file in place:

```bash
TOPIARY_LANGUAGE_DIR=queries topiary format --configuration config/languages.ncl -- test.gd
```

If you get an error that the idempotence check failed, it means that running the formatter a second time changed the already formatted file, which should ideally not happen. When iterating over a new feature, this is okay - you can first implement the feature, then run the formatter, and finally fix the idempotence issue.

You can use the `--skip-idempotence` flag to skip this check temporarily while developing new features:

```bash
TOPIARY_LANGUAGE_DIR=queries topiary format --configuration config/languages.ncl --skip-idempotence -- test.gd
```

### Running tests

To run the formatter's test suite, use this command:

```bash
cargo test
```

### Debugging and visualizing the tree structure

To visualize the graph structure that Topiary uses for formatting, you can use the `topiary visualise` command. It produces markup that you can pass to the open source program [Graphviz](https://graphviz.org/) to generate a visual representation of the abstract syntax tree (AST) used by Topiary.

This command generates the markup:

```bash
echo 'class Test:' | TOPIARY_LANGUAGE_DIR=queries topiary visualise --language gdscript --configuration config/languages.ncl
```

You can pipe the output to Graphviz's `dot` command to generate a vector image:

```bash
echo 'class Test:' | TOPIARY_LANGUAGE_DIR=queries topiary visualise --language gdscript --configuration config/languages.ncl | dot -Tsvg > image.svg
```

It will produce an SVG image like this:

![Example Graphviz graph generated from GDScript code](docs/images/example_graphviz_output.svg)

You can also use tree-sitter directly to parse GDScript files and visualize the concrete syntax tree. This shows you the raw structure that the tree-sitter parser generates, which you can then use to write formatting rules in Topiary:

```bash
tree-sitter parse --scope source.gdscript test.gd
```

This requires setting up tree-sitter on your computer and having the GDScript parser configured.

When you're getting started with contributing to the formatter, I recommend beginning with the tree visualization commands.

For example, if you want to add a formatting rule for function definitions, you'd first use these commands to see how the parser represents functions in the tree. Then you can write queries that target those specific nodes and apply formatting rules to them in `queries/gdscript.scm`.

## License

[MIT](https://github.com/GDQuest/godot-gdscript-formatter-tree-sitter/blob/main/LICENSE)

## Motivation

The Godot team has wanted an official GDScript formatter since the early days, but it has always been part of the engine's development backlog. It's a tool Godot users would use daily, so in 2022, we set out to sponsor the development of an [official GDScript formatter](https://github.com/godotengine/godot/pull/76211) built into Godot 4.

We put a lot of work into this project at GDQuest. Then, following the suggestion to break up the work into smaller contributions, a dedicated contributor, Scony, took over the project and tried breaking down the implementation [into small chunks](https://github.com/godotengine/godot/pull/97383) to make it much easier to review and merge. However, there isn't an active maintainer to review and merge the work, and the project has been stuck for a while now. The process looks like it will take a long time, and we need solutions we can work on quickly and use today.

Scony has been maintaining a solid set of community tools for GDScript, including a code formatter written in Python: [Godot GDScript Toolkit](https://github.com/Scony/godot-gdscript-toolkit). It's a great project that many Godot developers have used. So, why start another one?

The main reason to try this project is that Scony's formatter has grown quite complex over the years, and it has limitations for us at GDQuest that make it not work for our projects. Some of these limitations are also not easy to fix.

Since Scony made his great formatter, new technologies have come up that could make it much easier to build and maintain one: Tree Sitter and Topiary. This project started as a suggestion from one of Godot's core contributors to test these new technologies for GDScript formatting. While testing it, I could get results within just a couple of hours and found it to be a very promising approach that could lead to a simple and fast formatter that's relatively easy to maintain and extend for the community.

<details>
<summary>What is Tree-sitter?</summary>
Tree-sitter is a powerful parser generator that makes it easy to create programming language parsers that run natively (it generates C code). It gives you a simple query language to find patterns in the code and process them, a bit like CSS selectors (but simpler and more limited). It's used in modern code editors for syntax highlighting, code navigation, outline, folding, and more (Zed, Neovim, Emacs, Helix, Lapce...).
</details>

<details>
<summary>What is Topiary?</summary>
Topiary is a Rust library and command line program that makes it easy to build a code formatter using a parser based on Tree Sitter. You use the Tree Sitter query language to define how the code should be formatted, and Topiary handles the rest.
</details>
