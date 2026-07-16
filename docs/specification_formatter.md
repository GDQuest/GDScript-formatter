# GDScript formatter: Specification for line wrapping and horizontal packing

This document defines how the formatter should wrap long GDScript code.

It's a work in progress.

Main references:

- [Official GDScript style guide](https://docs.godotengine.org/en/stable/tutorials/scripting/gdscript/gdscript_styleguide.html)

## Definitions

### Flat layout

We say that a piece of code uses a *flat layout* when all its content stays on one line. Example:

```gdscript
var colors = [Color.RED, Color.GREEN, Color.BLUE]
```

### Broken layout

A piece of code uses a *broken layout* when its content uses several lines. Example:

```gdscript
var colors = [
	Color.RED,
	Color.GREEN,
	Color.BLUE,
]
```

### Packing

Packing is the choice of how many items or expression segments to place on each line.

### Vertical layout

A *vertical layout* is a layout where items in delimiters are broken across multiple lines, with each item starting on a new line after an opening parenthesis, bracket, or brace. Example:

```gdscript
spawn_character(
	position,
	direction,
)
```

### Construct

A *construct* is a syntactic element that groups multiple items or expression segments together. It could be a function call, a conditional statement, etc.

### Continuation layouts

The official GDScript style guide has a special case for vertical layouts called "continuation lines." In continuation lines, two indents are added to lines after the beginning of a construct.

In this document, we instead talk about a "continuation layout:" it's a layout that is different from and can override the vertical-item layout defined above. A continuation layout's lines do not each represent one item in a vertically broken comma-separated construct, although they may still appear inside delimiters (e.g. parentheses). This specification distinguishes three kinds of continuation layouts:

**Hanging continuation:** The first item stays on the line that opens the construct, and later items continue on following lines.

```gdscript
draw_string(font, position, text,
		alignment, width, font_size)
```

**Expression continuation:** One expression continues across several lines. The expression may be inside delimiters.

```gdscript
if (
		is_on_floor()
		and velocity.y >= 0.0
):
	land()
```

**Backslash continuation:** A backslash explicitly continues the statement on the next line.

```gdscript
create_tween() \
		.set_ease(Tween.EASE_OUT) \
		.set_trans(Tween.TRANS_BACK)
```

Anything other than these three continuation layouts is treated as a regular vertical layout and should not be subject to continuation layout rules.
