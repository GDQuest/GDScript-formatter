## This plug-in adds support for automatically formatting GDScript files on save
## and via a command in the Godot Editor, using the GDQuest GDScript Formatter.
##
## It also provides an option to install or update the formatter binary from the GitHub releases.
##
## See our website for more information on the formatter and how to use it:
## https://www.gdquest.com/library/gdscript_formatter/
@tool
extends EditorPlugin

const FormatterInstaller = preload("install_and_update.gd")
const FormatterMenu = preload("menu.gd")

const EDITOR_SETTINGS_CATEGORY = "gdquest_gdscript_formatter/"
const SETTING_FORMAT_ON_SAVE = "format_on_save"
const SETTING_SHORTCUT = "shortcut"
const SETTING_USE_SPACES = "use_spaces"
const SETTING_INDENT_SIZE = "indent_size"
const SETTING_REORDER_CODE = "reorder_code"
const SETTING_SAFE_MODE = "safe_mode"
const SETTING_FORMATTER_PATH = "formatter_path"
const SETTING_LINT_ON_SAVE = "lint_on_save"
const SETTING_LINT_LINE_LENGTH = "lint_line_length"
const SETTING_LINT_IGNORED_RULES = "lint_ignored_rules"

const COMMAND_PALETTE_CATEGORY = "gdquest gdscript formatter/"
const COMMAND_PALETTE_FORMAT_SCRIPT = "Format GDScript"
const COMMAND_PALETTE_LINT_SCRIPT = "Lint GDScript"
const COMMAND_PALETTE_INSTALL_UPDATE = "Install or Update Formatter"
const COMMAND_PALETTE_UNINSTALL = "Uninstall Formatter"
const COMMAND_PALETTE_REPORT_ISSUE = "Report Issue"

const DEFAULT_SETTINGS = {
	SETTING_FORMAT_ON_SAVE: false,
	SETTING_USE_SPACES: false,
	SETTING_INDENT_SIZE: 4,
	SETTING_REORDER_CODE: false,
	SETTING_SAFE_MODE: true,
	SETTING_FORMATTER_PATH: "",
	SETTING_LINT_ON_SAVE: false,
	SETTING_LINT_LINE_LENGTH: 100,
	SETTING_LINT_IGNORED_RULES: "",
}

## Which gutter lint icons are shown in.
## By default, gutter 0 is for breakpoints and 1 is for things like overrides.
const LINT_ICON_GUTTER := 2

var connection_list: Array[Resource] = []
var installer: FormatterInstaller = null
var formatter_cache_dir: String
var menu: FormatterMenu = null


func _init() -> void:
	for setting: String in DEFAULT_SETTINGS.keys():
		if not has_editor_setting(setting):
			set_editor_setting(setting, DEFAULT_SETTINGS[setting])

	if not has_editor_setting(SETTING_SHORTCUT):
		var default_shortcut := InputEventKey.new()
		default_shortcut.echo = false
		default_shortcut.pressed = true
		default_shortcut.keycode = KEY_I
		default_shortcut.ctrl_pressed = true
		default_shortcut.shift_pressed = false
		default_shortcut.alt_pressed = true

		var shortcut := Shortcut.new()
		shortcut.events.push_back(default_shortcut)

		set_editor_setting(SETTING_SHORTCUT, shortcut)


func _enter_tree() -> void:
	formatter_cache_dir = EditorInterface.get_editor_paths().get_cache_dir().path_join("gdquest")
	installer = FormatterInstaller.new(formatter_cache_dir)
	add_child(installer)
	installer.installation_completed.connect(
		func _on_installation_completed(binary_path: String) -> void:
			set_editor_setting(SETTING_FORMATTER_PATH, binary_path)
			add_format_command()
			remove_uninstall_command()
			add_uninstall_command()
			# After installing the formatter we can add the menu option to show the uninstall command
			if is_instance_valid(menu):
				menu.update_menu(true)
	)
	installer.installation_failed.connect(
		func _on_installation_failed(error_message: String) -> void:
			push_error("Formatter installation failed: ", error_message)
	)

	add_format_command()
	add_lint_command()
	add_install_update_command()
	add_uninstall_command()
	add_report_issue_command()

	menu = FormatterMenu.new()
	add_child(menu)
	menu.menu_item_selected.connect(_on_menu_item_selected)
	menu.update_menu(is_formatter_installed_locally())

	update_shortcut()
	resource_saved.connect(_on_resource_saved)


func _exit_tree() -> void:
	resource_saved.disconnect(_on_resource_saved)

	remove_format_command()
	remove_lint_command()
	remove_install_update_command()
	remove_uninstall_command()
	remove_report_issue_command()

	installer.queue_free()
	installer = null

	if is_instance_valid(menu):
		menu.menu_item_selected.disconnect(_on_menu_item_selected)
		menu.remove_formatter_menu()
		menu.queue_free()
		menu = null


func _shortcut_input(event: InputEvent) -> void:
	if not has_command(get_editor_setting(SETTING_FORMATTER_PATH)):
		return
	var shortcut := get_editor_setting(SETTING_SHORTCUT) as Shortcut
	if not is_instance_valid(shortcut):
		return
	if shortcut.matches_event(event) and event.is_pressed() and not event.is_echo():
		if format_current_script():
			get_tree().root.set_input_as_handled()


func format_current_script() -> bool:
	if not EditorInterface.get_script_editor().is_visible_in_tree():
		return false
	var current_script := EditorInterface.get_script_editor().get_current_script()
	if not is_instance_valid(current_script) or not current_script is GDScript:
		return false
	var code_edit: CodeEdit = EditorInterface.get_script_editor().get_current_editor().get_base_editor()

	var formatted_code := format_code(current_script)
	if formatted_code.is_empty():
		return false

	reload_code_edit(code_edit, formatted_code)
	return true


func lint_current_script() -> bool:
	if not EditorInterface.get_script_editor().is_visible_in_tree():
		return false

	var current_script := EditorInterface.get_script_editor().get_current_script()
	if not is_instance_valid(current_script) or not current_script is GDScript:
		return false

	var code_edit: CodeEdit = EditorInterface.get_script_editor().get_current_editor().get_base_editor()

	var lint_issues := lint_code(current_script)
	if lint_issues.is_empty():
		print("No linting issues found.")
		clear_lint_highlights(code_edit)
		return true

	apply_lint_highlights(code_edit, lint_issues)
	print_lint_summary(lint_issues, current_script.resource_path)

	return true


func update_shortcut() -> void:
	for obj: Resource in connection_list:
		obj.changed.disconnect(update_shortcut)

	connection_list.clear()

	var shortcut := get_editor_setting(SETTING_SHORTCUT) as Shortcut
	if is_instance_valid(shortcut):
		for event: InputEvent in shortcut.events:
			if is_instance_valid(event):
				event.changed.connect(update_shortcut)
				connection_list.push_back(event)

	remove_format_command()
	add_format_command()


func _on_resource_saved(saved_resource: Resource) -> void:
	if saved_resource is not GDScript:
		return

	var format_on_save := get_editor_setting(SETTING_FORMAT_ON_SAVE) as bool
	var lint_on_save := get_editor_setting(SETTING_LINT_ON_SAVE) as bool

	if not format_on_save and not lint_on_save:
		return

	var script := saved_resource as GDScript

	if not has_command(get_editor_setting(SETTING_FORMATTER_PATH)) or not is_instance_valid(script):
		return

	if format_on_save:
		var formatted_code := format_code(script, false)
		if formatted_code.is_empty():
			return

		script.source_code = formatted_code
		ResourceSaver.save(script)
		script.reload()

		var script_editor := EditorInterface.get_script_editor()
		var open_script_editors := script_editor.get_open_script_editors()
		var open_scripts := script_editor.get_open_scripts()

		if not open_scripts.has(script):
			return

		if script_editor.get_current_script() == script:
			reload_code_edit(script_editor.get_current_editor().get_base_editor(), formatted_code, true)
		elif open_scripts.size() == open_script_editors.size():
			for i: int in range(open_scripts.size()):
				if open_scripts[i] == script:
					reload_code_edit(open_script_editors[i].get_base_editor(), formatted_code, true)
					return
		else:
			push_error("GDScript Formatter error: Unknown situation, can't reload code editor in Editor. Please report this issue.")

	if lint_on_save:
		var code_edit: CodeEdit = EditorInterface.get_script_editor().get_current_editor().get_base_editor()
		var lint_issues := lint_code(script)
		if lint_issues.is_empty():
			clear_lint_highlights(code_edit)
		else:
			apply_lint_highlights(code_edit, lint_issues)
			print_lint_summary(lint_issues, script.resource_path)


func add_format_command() -> void:
	if not has_command(get_editor_setting(SETTING_FORMATTER_PATH)):
		push_error(
			'GDScript Formatter: The command "%s" can\'t be found in your environment.\n' % get_editor_setting(SETTING_FORMATTER_PATH) +
			'\tIf you have not installed the formatter, use the install/update command from the command palette.\n' +
			'\tIf you have installed the formatter, change "formatter_path" to a valid command in the "GDScript Formatter" section in Editor Settings.',
		)
		return
	var shortcut := get_editor_setting(SETTING_SHORTCUT) as Shortcut
	EditorInterface.get_command_palette().add_command(
		COMMAND_PALETTE_FORMAT_SCRIPT,
		COMMAND_PALETTE_CATEGORY + COMMAND_PALETTE_FORMAT_SCRIPT,
		format_current_script,
		shortcut.get_as_text() if is_instance_valid(shortcut) else "None",
	)


func remove_format_command() -> void:
	EditorInterface.get_command_palette().remove_command(COMMAND_PALETTE_CATEGORY + COMMAND_PALETTE_FORMAT_SCRIPT)


func add_lint_command() -> void:
	if not has_command(get_editor_setting(SETTING_FORMATTER_PATH)):
		return

	EditorInterface.get_command_palette().add_command(
		COMMAND_PALETTE_LINT_SCRIPT,
		COMMAND_PALETTE_CATEGORY + COMMAND_PALETTE_LINT_SCRIPT,
		lint_current_script,
	)


func remove_lint_command() -> void:
	EditorInterface.get_command_palette().remove_command(
		COMMAND_PALETTE_CATEGORY + COMMAND_PALETTE_LINT_SCRIPT,
	)


func add_install_update_command() -> void:
	EditorInterface.get_command_palette().add_command(
		COMMAND_PALETTE_INSTALL_UPDATE,
		COMMAND_PALETTE_CATEGORY + COMMAND_PALETTE_INSTALL_UPDATE,
		installer.install_or_update_formatter,
	)


func remove_install_update_command() -> void:
	EditorInterface.get_command_palette().remove_command(COMMAND_PALETTE_CATEGORY + COMMAND_PALETTE_INSTALL_UPDATE)


func add_uninstall_command() -> void:
	if is_formatter_installed_locally():
		EditorInterface.get_command_palette().add_command(
			COMMAND_PALETTE_UNINSTALL,
			COMMAND_PALETTE_CATEGORY + COMMAND_PALETTE_UNINSTALL,
			uninstall_formatter,
		)


func remove_uninstall_command() -> void:
	EditorInterface.get_command_palette().remove_command(COMMAND_PALETTE_CATEGORY + COMMAND_PALETTE_UNINSTALL)


func add_report_issue_command() -> void:
	EditorInterface.get_command_palette().add_command(
		COMMAND_PALETTE_REPORT_ISSUE,
		COMMAND_PALETTE_CATEGORY + COMMAND_PALETTE_REPORT_ISSUE,
		report_issue,
	)


func remove_report_issue_command() -> void:
	EditorInterface.get_command_palette().remove_command(COMMAND_PALETTE_CATEGORY + COMMAND_PALETTE_REPORT_ISSUE)


func is_formatter_installed_locally() -> bool:
	var binary_name := "gdscript-formatter"
	if OS.get_name().to_lower().contains("windows"):
		binary_name = "gdscript-formatter.exe"
	var binary_path := formatter_cache_dir.path_join(binary_name)
	return FileAccess.file_exists(binary_path)


func uninstall_formatter() -> void:
	var binary_name := "gdscript-formatter"
	if OS.get_name().to_lower().contains("windows"):
		binary_name = "gdscript-formatter.exe"
	var binary_path := formatter_cache_dir.path_join(binary_name)

	if FileAccess.file_exists(binary_path):
		DirAccess.remove_absolute(binary_path)
		print("GDScript formatter uninstalled successfully from: ", binary_path)
		set_editor_setting(SETTING_FORMATTER_PATH, DEFAULT_SETTINGS[SETTING_FORMATTER_PATH])

		remove_format_command()
		add_format_command()
		remove_uninstall_command()
		add_uninstall_command()
		if is_instance_valid(menu):
			menu.update_menu(false)
	else:
		push_error("GDScript formatter not found in cache directory: ", binary_path)


func reorder_code() -> bool:
	if not EditorInterface.get_script_editor().is_visible_in_tree():
		return false
	var current_script := EditorInterface.get_script_editor().get_current_script()
	if not is_instance_valid(current_script) or not current_script is GDScript:
		return false
	var code_edit: CodeEdit = EditorInterface.get_script_editor().get_current_editor().get_base_editor()

	var formatted_code := format_code(current_script, true)
	if formatted_code.is_empty():
		return false

	reload_code_edit(code_edit, formatted_code)
	return true


func report_issue() -> void:
	OS.shell_open("https://github.com/GDQuest/GDScript-formatter/issues")


func show_help() -> void:
	OS.shell_open("https://www.gdquest.com/library/gdscript_formatter/")


func _on_menu_item_selected(command: String) -> void:
	match command:
		"format_script":
			format_current_script()
		"lint_script":
			lint_current_script()
		"reorder_code":
			reorder_code()
		"install_update":
			installer.install_or_update_formatter()
		"uninstall":
			uninstall_formatter()
		"report_issue":
			report_issue()
		"help":
			show_help()
		_:
			push_warning("Unsupported command sent from the menu: " + command)


func has_command(command: String) -> bool:
	var output: Array = []
	var exit_code := OS.execute(command, ["--version"], output)
	return exit_code == OK


## Reloads the code editor with new text while preserving editor state.
## This includes cursor position, scroll position, breakpoints, bookmarks, and folds.
func reload_code_edit(
		code_edit: CodeEdit,
		new_text: String,
		tag_saved := false,
) -> void:
	var editor_state := CodeEditState.new(code_edit)
	code_edit.text = new_text
	if tag_saved:
		code_edit.tag_saved_version()
	editor_state.restore_to_editor(code_edit)
	code_edit.update_minimum_size()
	code_edit.text_changed.emit()


func get_editor_setting(setting_name: String) -> Variant:
	var editor_settings := EditorInterface.get_editor_settings()
	var full_setting_key := EDITOR_SETTINGS_CATEGORY + setting_name
	if editor_settings.has_setting(full_setting_key):
		return editor_settings.get_setting(full_setting_key)
	return DEFAULT_SETTINGS[setting_name]


func set_editor_setting(setting_name: String, value: Variant) -> void:
	var editor_settings := EditorInterface.get_editor_settings()
	var full_setting_key := EDITOR_SETTINGS_CATEGORY + setting_name
	editor_settings.set_setting(full_setting_key, value)


func has_editor_setting(setting_name: String) -> bool:
	var editor_settings := EditorInterface.get_editor_settings()
	var full_setting_key := EDITOR_SETTINGS_CATEGORY + setting_name
	return editor_settings.has_setting(full_setting_key)


## Formats a GDScript file using the GDScript Formatter,
## and returns the formatted code as a string. Optionally reorders the code.
func format_code(script: GDScript, force_reorder := false) -> String:
	var script_path := script.resource_path
	if script_path.is_empty():
		push_error("GDScript Formatter Error: Can't format an unsaved script.")
		return ""

	# This is a bit of a hack to avoid two issues:
	#
	# 1. Running GDScript formatter on stdin/stdout through Godot with
	# OS.execute() has encoding issues with UTF-8 characters and we don't have
	# control over the output encoding (it might be assuming ASCII characters)
	#
	# 2. If we modify a file in place using the external formatter from Godot,
	# it will bring up a pop-up that warns users that the file has been changed
	# outside Godot.
	#
	# To work around that, I save a copy of the script as a temporary file,
	# format the file, and read it specifically as a UTF-8 string.
	var source_file := FileAccess.open(ProjectSettings.globalize_path(script_path), FileAccess.READ)
	if not source_file:
		push_error("GDScript Formatter Error: Cannot read source file: " + script_path)
		return ""

	# FileAccess.get_as_text() reads the file as UTF-8. We use it here and after
	# formatting the temporary file.
	var source_content := source_file.get_as_text()
	source_file.close()

	var path_temporary_file := OS.get_temp_dir().path_join("gdscript_formatter_%d.gd" % Time.get_ticks_msec())
	var temporary_file := FileAccess.open(path_temporary_file, FileAccess.WRITE)
	if temporary_file == null:
		push_error("GDScript Formatter Error: Cannot create temporary file: " + path_temporary_file)
		return ""
	temporary_file.store_string(source_content)
	temporary_file.close()

	var formatter_arguments := PackedStringArray()
	if get_editor_setting(SETTING_USE_SPACES):
		formatter_arguments.push_back("--use-spaces")
		formatter_arguments.push_back("--indent-size=%d" % get_editor_setting(SETTING_INDENT_SIZE))

	var should_reorder := force_reorder or get_editor_setting(SETTING_REORDER_CODE) as bool

	if should_reorder:
		formatter_arguments.push_back("--reorder-code")

	if get_editor_setting(SETTING_SAFE_MODE):
		formatter_arguments.push_back("--safe")

	formatter_arguments.push_back(path_temporary_file)

	var output: Array = []
	var exit_code := OS.execute(
		get_editor_setting(SETTING_FORMATTER_PATH),
		formatter_arguments,
		output,
	)

	var formatted_content := ""
	if exit_code == OK:
		var result_file := FileAccess.open(path_temporary_file, FileAccess.READ)
		if result_file:
			formatted_content = result_file.get_as_text()
			result_file.close()
		else:
			push_error("Format GDScript: Cannot read formatted output from temp file")
	else:
		push_error("Format GDScript failed: " + script_path)
		push_error(
			"\tExit code: " + str(exit_code) + " Output: " +
			(output[0].strip_edges() if output.size() > 0 else "No output"),
		)
		push_error('\tIf your script does not have any syntax errors, this might be a formatter bug.')

	if FileAccess.file_exists(path_temporary_file):
		DirAccess.remove_absolute(path_temporary_file)

	return formatted_content


## Lints a GDScript file using the GDScript Formatter's linter,
## and returns an array of lint issues.
func lint_code(script: GDScript) -> Array:
	var script_path := script.resource_path
	var output: Array = []
	var formatter_arguments: Array = ["lint", ProjectSettings.globalize_path(script_path)]

	var exit_code := OS.execute(get_editor_setting(SETTING_FORMATTER_PATH), formatter_arguments, output)
	if exit_code == OK:
		return [] # No issues found

	if exit_code == 1:
		# Parse lint output - the output is a single string with multiple lines
		var issues = []
		for output_item in output:
			var lines = output_item.split("\n")
			for line in lines:
				var trimmed_line = line.strip_edges()
				if trimmed_line.is_empty():
					continue
				var issue = parse_lint_issue(trimmed_line)
				if issue != null and not issue.is_empty():
					issues.push_back(issue)
		return issues

	push_error("Lint GDScript failed: " + script_path)
	push_error("\tExit code: " + str(exit_code) + " Output: " + (output.front().strip_edges() if output.size() > 0 else "No output"))
	return []


## Parses a lint issue line and returns a dictionary with issue information
func parse_lint_issue(line: String) -> Dictionary:
	# Expected format: filename:line:rule:severity: message
	var parts = line.split(":", 4)
	if parts.size() < 5:
		return { }

	return {
		"line": int(parts[1]) - 1, # Convert to 0-based indexing
		"rule": parts[2],
		"severity": parts[3],
		"message": parts[4].strip_edges(),
	}


## Applies lint highlighting to the code editor
func apply_lint_highlights(code_edit: CodeEdit, issues: Array) -> void:
	clear_lint_highlights(code_edit)

	for issue in issues:
		var line_number: int = issue.line
		var severity: String = issue.severity

		# Set line background color based on severity
		var color: Color
		if severity == "error":
			color = Color(1, 0, 0, 0.1)
		else: # warning
			color = Color(1, 1, 0, 0.1)

		code_edit.set_line_background_color(line_number, color)

		# Add gutter icon for severity
		var icon_name = "StatusError" if severity == "error" else "StatusWarning"
		var icon = EditorInterface.get_editor_theme().get_icon(icon_name, "EditorIcons")
		code_edit.set_gutter_type(LINT_ICON_GUTTER, CodeEdit.GutterType.GUTTER_TYPE_ICON)
		code_edit.set_line_gutter_icon(line_number, LINT_ICON_GUTTER, icon)


## Prints a detailed summary of lint issues to the output
func print_lint_summary(issues: Array, script_path: String) -> void:
	print_rich("\n[b]=== Linting Results for %s ===[/b]\n" % script_path)
	print_rich("[b]Found [i]%s[/i] issue(s)\n[/b]" % issues.size())

	for issue in issues:
		var line_display = str(issue.line + 1) # Convert back to 1-based for display
		var severity_label = issue.severity.to_upper()
		print_rich(
			"[color=%s]%s[/color] on line [color=cyan]%s[/color] ([i]%s[/i])" % [
				"red" if severity_label == "ERROR" else "yellow",
				severity_label,
				line_display,
				issue.rule,
			],
		)
		print_rich("[i]%s[/i]\n" % [issue.message])

	print_rich("[b]=== End Linting Results ===[/b]\n")


## Clears all lint highlighting from the code editor
func clear_lint_highlights(code_edit: CodeEdit) -> void:
	for line in range(code_edit.get_line_count()):
		code_edit.set_line_background_color(line, Color(0, 0, 0, 0))
		code_edit.set_line_gutter_icon(line, LINT_ICON_GUTTER, null)


## Data structure to hold code editor state information
class CodeEditState:
	var caret_line: int
	var caret_column: int
	var horizontal_scroll: int
	var vertical_scroll: int
	var breakpoints: Dictionary[int, String] = { }
	var bookmarks: Dictionary[int, String] = { }
	var folds: Dictionary[int, String] = { }
	var code_edit: CodeEdit


	func _init(code_edit: CodeEdit) -> void:
		self.code_edit = code_edit
		caret_line = code_edit.get_caret_line()
		caret_column = code_edit.get_caret_column()
		horizontal_scroll = code_edit.scroll_horizontal
		vertical_scroll = code_edit.scroll_vertical

		for line in code_edit.get_breakpointed_lines():
			breakpoints[line] = code_edit.get_line(line)
		for line in code_edit.get_bookmarked_lines():
			bookmarks[line] = code_edit.get_line(line)
		for line in code_edit.get_folded_lines():
			folds[line] = code_edit.get_line(line)


	func restore_to_editor(code_edit: CodeEdit) -> void:
		var new_line_count := code_edit.get_line_count()

		_restore_line_features(breakpoints, code_edit.set_line_as_breakpoint, new_line_count)
		_restore_line_features(bookmarks, code_edit.set_line_as_bookmarked, new_line_count)
		_restore_line_features(folds, func(line: int, _is_folded: bool) -> void: code_edit.fold_line(line), new_line_count)

		code_edit.set_caret_line(caret_line)
		code_edit.set_caret_column(caret_column)

		code_edit.scroll_horizontal = horizontal_scroll
		code_edit.scroll_vertical = vertical_scroll


	## Restores line-based features (breakpoints, bookmarks, folds) by finding the best matching lines
	## in the new text based on similarity to the original line text.
	##
	## Big thanks to https://github.com/Daylily-Zeleen/GDScript-Formatter for this approach.
	func _restore_line_features(
			stored_features: Dictionary,
			set_line_func: Callable,
			new_line_count: int,
	) -> void:
		var stored_lines := PackedInt64Array(stored_features.keys())
		for line_index in range(stored_lines.size()):
			var original_line := stored_lines[line_index] as int
			var original_text := stored_features[original_line] as String

			# After formatting lines can move, so we need to find the best match for the original line
			# to restore the breakpoints, bookmarks, and folds.
			# We first check the same line, then we expand our search outwards until we find a match.
			# We use a similarity threshold of 0.9 to account for minor changes in the line text.
			# This should be sufficient for most cases, but might need adjustment for edge cases.
			# If no match is found, we skip restoring this feature
			if original_line < new_line_count and code_edit.get_line(original_line).similarity(original_text) > 0.9:
				set_line_func.call(original_line, true)
				continue

			var line_above := original_line - 1
			var line_below := original_line + 1
			while line_above >= 0 or line_below < new_line_count:
				if line_below < new_line_count and code_edit.get_line(line_below).similarity(original_text) > 0.9:
					set_line_func.call(line_below, true)
					break
				if line_above >= 0 and code_edit.get_line(line_above).similarity(original_text) > 0.9:
					set_line_func.call(line_above, true)
					break

				line_above -= 1
				line_below += 1
