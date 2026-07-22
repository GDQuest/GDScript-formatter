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

## 1. General Rules

### 1.1. Line width is a loose target

The formatter should keep code at or below `max_line_length` when it can safely add a line break.

The max line length is not a strict limit, though. Strings, node path, identifiers, values, or comments are indivisible content and may cause a line to remain longer than the target.

```gdscript
const DOCUMENTATION_URL = "https://example.com/of/a/url/that/cannot/be/split/even/when/exceeding/max/line/length"
```

All parts of the codebase including the renderer and linter should use the same, consistent width measurement. The measurement should account for the current visual column, indentation, tabs, Unicode width, spaces added by the formatter, commas, backslashes, and more generally any characters inserted by the formatter.

### 1.2. Prefer flat layout when the code fits one line

The formatter should use flat layout when a construct fits and no syntax or comment requires a line break.

Input:

```gdscript
var names = [
	"Ana",
	"Bao",
	"Chidi",
]
```

Output:

```gdscript
var names = ["Ana", "Bao", "Chidi"]
```

Note: This is a change from the previous implementation of the formatter. The previous formatter preserved many source line breaks because it did not wrap lines automatically (it used line breaks as a hint to lay lines vertically). This new formatter should be as hands-off as possible: it should wrap lines when needed and merge them again when the user removes something and the code fits on one line.

The following cases may stay multiline even when their text would fit as a single line:

- Enums, because the official style guide requires one member per line.
- Constructs that contain comments that need to be on separate lines.
- Code that includes an unavoidable hard line break, like a multiline string or a lambda with a multiline body.
- Code inside a `# fmt: off` region.

### 1.3. Existing line breaks do not force broken layout

In the first pass of the formatter, source line breaks are not formatting instructions.

Input:

```gdscript
call(
	first,
	second,
)
```

Output:

```gdscript
call(first, second)
```

## 2. Indentation

### 2.1. Vertical items use one extra indentation level

Items in vertical layout use one level more than the line that opens the construct.

```gdscript
func move_character():
	update_velocity(
		input_direction,
		maximum_speed,
		acceleration,
	)
```

This rule applies to arrays, dictionaries, parameters, arguments, annotation arguments, and other comma-separated items inside delimiters.

```gdscript
enum Element {
	EARTH,
	WATER,
	AIR,
	FIRE,
}
var party = [
	"Godot",
	"Godette",
	"Steve",
]
var character = {
	"name": "Bob",
	"job": "Mechanic",
}


func configure_character():
	var settings = {
		"speed": 300.0,
		"acceleration": 1200.0,
	}
```

### 2.2. Continuation layouts use two extra indentation levels

Continuation layouts use two extra indentation levels. Conditionals:

```gdscript
if (
		position.x > 200 and position.x < 400
		and position.y > 300 and position.y < 400
):
	pass
```

Backslash continuations:

```gdscript
var total = base_value \
		+ equipment_bonus \
		+ status_bonus
```

This would also apply to the following case (arguments continuing on hanging lines), although the formatter currently does not produce this kind of packing:

```gdscript
effect.interpolate_property(sprite, "transform/scale",
		sprite.get_scale(), Vector2(2.0, 2.0), 0.3,
		Tween.TRANS_QUAD, Tween.EASE_OUT)
```

## 3. Delimiters and comma-separated constructs

### 3.1. Break after the opening delimiter

When a construct uses broken layout with delimiters, its first item should start on the next line.

```gdscript
effect.interpolate_property(
	sprite,
	"transform/scale",
	sprite.get_scale(),
	Vector2(2.0, 2.0),
	0.3,
	Tween.TRANS_QUAD,
	Tween.EASE_OUT,
)
```

The formatter should normalize hanging input to this vertical layout when the construct must wrap.

### 3.1.1. Keep a fitting call prefix before broken arguments

Method-call arguments own their layout independently from the attribute or method chain that contains the call. When the receiver, method name, and opening parenthesis fit on the current line, the formatter should keep them there even when the arguments require broken layout.

```gdscript
_view_overlay.gui_input.connect(
	func(event: InputEvent) -> void:
		_panel_gui_input(event)
		_view_overlay.mouse_default_cursor_shape = _bubble_container.mouse_default_cursor_shape
)
```

In particular, a multiline lambda argument or ordinary overlong argument list breaks its nearest argument list after the opening parenthesis. It must not by itself break the enclosing attribute chain at every dot.

```gdscript
f.args.map(
	func(arg: Dictionary) -> Dictionary:
		arg.erase("name")
		return arg
)
```

```gdscript
mob.velocity = mob.velocity.move_toward(
	desired_velocity,
	velocity_distance * acceleration_factor * delta
)
```

### 3.2. Put the closing delimiter on its own line

The closing parenthesis, bracket, or brace of a broken construct should always be on its own line.

```gdscript
var colors = [
	Color.RED,
	Color.GREEN,
]
```

**Do not** produce this layout:

```gdscript
var colors = [
	Color.RED,
	Color.GREEN,]
```

### 3.3. Put one item on each line by default

Arrays, dictionaries, function parameters, function arguments, annotation arguments, and similar comma-separated constructs should use one item per line when broken by default. Examples:

```gdscript
var values = [
	10,
	20,
	30,
	40,
]
```

```gdscript
func spawn_character(
	position: Vector2,
	direction: Vector2,
	speed: float,
) -> CharacterBody2D:
	pass
```

```gdscript
spawn_character(
	spawn_position,
	Vector2.RIGHT,
	300.0,
)
```

```gdscript
@export_custom(
	PROPERTY_HINT_RANGE,
	"0,100,1",
)
var health := 100
```

```gdscript
# For dictionaries, put one key-value pair per line when broken.
var character = {
	"name": "Bob",
	"age": 27,
	"job": "Mechanic",
}
```

### 3.4. Always format enums vertically

Every enum member should be on its own line, even when the enum would fit on one line.

Input:

```gdscript
enum Element { EARTH, WATER, AIR, FIRE }
```

Output:

```gdscript
enum Element {
	EARTH,
	WATER,
	AIR,
	FIRE,
}
```

This rule follows the official GDScript style guide.

### 3.5. Add a trailing comma to multiline constructs

The formatter should add a trailing comma to every multiline comma-separated construct where the GDScript grammar accepts it.

```gdscript
call(
	first,
	second,
)
```

```gdscript
func combine(
	first: String,
	second: String,
) -> String:
	return first + second
```

The formatter should remove the trailing comma when the construct collapses to one line.

```gdscript
call(first, second)
```

**Exception 1:** The `preload()` function must not have a trailing comma; GDScript does not allow it:

```gdscript
const MUSIC = preload(
	"res://audio/a_very_long_music_file_name.ogg"
)
```
