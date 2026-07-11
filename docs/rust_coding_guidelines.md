# Rust coding guidelines

These are the coding guidelines for the Rust code in the GDScript formatter codebase. They're designed to keep the rust code accessible to non-rust developers and keep the code relatively straightforward so it's more or less easily translatable into C++ should the need arise in the future.

We want to keep the code simple (meaning, avoid adding too much abstraction, indirection, and dependencies) and fast enough (as this program is used to stress test and improve the tree sitter parser for GDScript, it constrains the performance we can get; the parse dominates the formatting performance even if the program takes 1ms or less for most GDScript files).

In short, the code should be procedural, avoid unnecessary abstractions, and avoid hiding too much control flow. We also want to avoid excessive memory allocations and other unnecessary operations in the code we control.

## Guidelines

### Favor inlining code and avoid indirection

Prefer inlining code over offloading to single-use functions. Extracting large code chunks that form a meaningful unit (e.g. `parse_function_body()` or `format_ternary_expression()`) into separate functions is OK. But avoid extracting short pieces of code as code is easier to read when it reads top to bottom like a book. We want the code to require little jumps to understand the logic whenever possible. Split code to a function when:

- The exact same code is repeated more than twice/needs to be executed multiple times
- The code is complex or hard to understand and benefits from being split into a separate function
- The code is a large semantic unit of work that reads well by itself

The purpose of this guideline is to limit indirection. Also, avoid introducing wrapper functions around code modules, wrapping types, casting types; refactor instead if possible.

### Write plain loops, avoid iterators

Prefer `for` and `while` loops over rust functional-style iterator chains. This is so that someone coming from another programming language is more likely to understand the code and be able to contribute.

Favor code like this:

```rust
let mut uses_extends_keyword = false;
for current_index in 0..node.named_child_count() {
    let current_child = node.named_child(current_index);
    if let Some(current_child) = current_child {
        if kind_of(current_child) == GDScriptNodeKind::Extends {
            uses_extends_keyword = true;
            break;
        }
    }
}
```

Avoid code like this:

```rust
let uses_extends_keyword = (0..node.named_child_count())
    .filter_map(|current_index| node.named_child(current_index as u32))
    .any(|current_child| kind_of(current_child) == GDScriptNodeKind::Extends);
```

### Allocate upfront when you know the size (or you can estimate)

When building a `Vec` where the final size is known or you can make a reasonable estimate, call `Vec::with_capacity` to pre-allocate a buffer.

```rust
let mut out = Vec::with_capacity(node.named_child_count());
```

### Avoid closures

If a check is used in one place, write it inline as a simple condition. If a check or piece of code is reused across three or more places, extract a simple function with a name that says what it tests. Example:

```rust
fn detect_if_needs_two_blank_lines(kind: GDScriptNodeKind) -> bool {
    matches!(kind, GDScriptNodeKind::Function | GDScriptNodeKind::ClassDefinition)
}
```

Exception: closures are fine for `ok_or_else(|| ...)` to defer error string
construction, and for test helpers that need to capture local state.

### Avoid defensive code

Avoid early returns that prevent running functions when the data passed to a function is invalid. We don't want to silently avoid failures. When making a program like this one, the user is not going to use it as a library, so we want to raise as many errors as possible if we make a mistake with the code (in particular, compile-time errors).

### Use explicit structs over tuples

Rust lets you return tuples of values from functions. Always prefer defining a struct to have an explicit typed data structure over using tuples. This makes the code easier to read, and it makes it easier for us to document, autocomplete, and generally know what we're working with.

### Use clear names and avoid abbreviations

Use descriptive names that convey what the function or variable does. Avoid abbreviations unless in rare cases where the abbreviation is widely used in rust codebases (e.g. `map_err(|e| ...)`)

Names like `current_index` or `current_child_index` are clearer for everyone than a generic `i` or `ci`, they remove cognitive load from the code or the need for extra comments that can fall out of sync with the code.

It's better for the code to look a bit more verbose and read like a book than to be terse and harder to read and debug.

### Use early returns over deep nesting

Return or `continue` as soon as you know you can. Avoid indenting and nesting code blocks deeply when possible.

```rust
// Good
if node.child_count() == 0 {
    emit_leaf(node, docs);
    return;
}
// rest of function at indent level 0

// Avoid
if node.child_count() == 0 {
    emit_leaf(node, docs);
} else {
    // whole function indented
}
```

### Avoid having many function parameters

Try to keep function parameters to a minimum. If you need many parameters, consider using a struct instead. Especially if the parameter count may grow in the future.

### Use `matches!` but avoid other macros

`matches!` reads like a sentence: "if the kind matches Function or ClassDef".
It is common enough in Rust codebases that non-Rust readers will learn it once
and see it everywhere. Avoid other macros in control flow.

### Put Mutable output parameter at the end

Functions that build something take `&mut Vec<...>` as their last parameter.
This is a soft convention from the existing code, keep it.

```rust
fn build_node(input: &ParseInput, node: Node, docs: &mut Vec<RenderToken>);
```

### Avoid trait objects and dynamic dispatch

Avoid using heap-allocated things like `Box<dyn Trait>`, and avoid the use of `dyn` (i.e., trait objects used for dynamic dispatch/polymorphism at runtime). Favor static dispatch like using an enum with a match statement. See formatter.rs for an example.

### Comments should mainly explain why, not what

If the code does something that's not obvious, explain the reason. Do not paraphase the code: assume that the reader or contributor can read code.

Comments that briefly summarize what a long, complex function or block of code does are okay too as they help outline the overall purpose and flow of the code. For those kinds of comments, favor putting them in docstrings for smaller functions as an explanation of what the function does (up to 100 lines). You can use comments in the code body for larger or complex functions.

### Avoid external dependencies whenever possible

At the beginning of development we used a few libraries to accelerate production of the program's first release. However some popular robust libraries, while probably useful in really complex programs, are heavy for a simple command line program like this one. They make compilation slower for little gain. We replaced `clap` (command line argument parsing) and `rayon` (multi-threading toolkit) with our own code and it should stay this way.

Favor vendoring code over external, moving dependencies. Use/vendor libraries when we need to support, for example, a specific file format or protocol that has an official and complete implementation.

## Not goals

Do not strive to do any of these when contributing:

- Minimize line count. Four clear lines of imperative code with clear variable names beat one dense line with functional-style code and one-letter variables.
- Make everything pure. Don't be afraid of mutating buffers or passing mutable references around.
- Abstract or prepare for future requirements that do not exist yet. Outside of the formatter's core architecture, which does need to help us maintain the formatter and account for mid-term needs, we want to focus on implementing concrete solutions to verified problems that multiple users have. The solutions we choose to implement should always be rooted in user experience.
