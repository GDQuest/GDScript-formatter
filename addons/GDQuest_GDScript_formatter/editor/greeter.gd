@tool
class_name Greeter
extends Window

signal action_pressed(action: String)
signal setting_changed(setting: String, value: Variant)

@onready var _addon_version_label: Label = $Container/Layout/Versions/Addon/Layout/Meta/Value
@onready var _formatter_version_label: Label = $Container/Layout/Versions/Formatter/Layout/Meta/Value
@onready var _heading_label: Label = $Container/Layout/Heading/Description

@onready var _docs_button: Button = $Container/Layout/SplitPanel/Resources/Items/Docs/HBoxContainer/DocsButton
@onready var _issues_button: Button = $Container/Layout/SplitPanel/Resources/Items/Issues/HBoxContainer/IssuesButton
@onready var _formatter_button: Button = $Container/Layout/Versions/Formatter/Layout/FormatterButton
@onready var _addon_button: Button = $Container/Layout/Versions/Addon/Layout/AddonButton
@onready var _format_on_save_toggle: CheckButton = $Container/Layout/SplitPanel/QuickSettings/Items/FormatOnSave/Toggle
@onready var _lint_on_save_toggle: CheckButton = $Container/Layout/SplitPanel/QuickSettings/Items/LintOnSave/Toggle

var _addon_version: String = "-"
var _formatter_version: String = "-"
var _setting_states: Dictionary = { }


func _ready() -> void:
	_addon_version_label.text = _addon_version
	_formatter_version_label.text = _formatter_version

	close_requested.connect(_handle_close)

	_docs_button.button_down.connect(_handle_docs_press)
	_issues_button.button_down.connect(_handle_issues_press)
	_formatter_button.button_down.connect(_handle_formatter_press)
	_addon_button.button_down.connect(_handle_addon_press)
	_format_on_save_toggle.toggled.connect(_handle_format_on_save_toggle)
	_lint_on_save_toggle.toggled.connect(_handle_lint_on_save_toggle)

	_update_tagline()
	_update_setting_ui()


func set_addon_version(version: String) -> void:
	_addon_version = version

	if is_node_ready():
		_addon_version_label.text = _addon_version


func set_formatter_version(version: String) -> void:
	_formatter_version = version

	if is_node_ready():
		_formatter_version_label.text = _formatter_version


func set_setting_state(setting: String, value: Variant) -> void:
	_setting_states.set(setting, value)

	if is_node_ready():
		_update_setting_ui()


func _update_tagline() -> void:
	var godot_version = Engine.get_version_info()
	var version_major = str(godot_version.major)

	_heading_label.text = _heading_label.text.replace("<MAJOR>", version_major)


func _update_setting_ui() -> void:
	if _setting_states.has("format_on_save"):
		var state = _setting_states.get("format_on_save")

		_format_on_save_toggle.button_pressed = state == true

	if _setting_states.has("lint_on_save"):
		var state = _setting_states.get("lint_on_save")

		_lint_on_save_toggle.button_pressed = state == true


func _handle_close() -> void:
	close_requested.disconnect(_handle_close)

	_docs_button.button_down.disconnect(_handle_docs_press)
	_issues_button.button_down.disconnect(_handle_issues_press)
	_formatter_button.button_down.disconnect(_handle_formatter_press)
	_addon_button.button_down.disconnect(_handle_addon_press)

	hide()


func _handle_docs_press() -> void:
	action_pressed.emit("help")


func _handle_issues_press() -> void:
	action_pressed.emit("report_issue")


func _handle_formatter_press() -> void:
	action_pressed.emit("install_update")


func _handle_addon_press() -> void:
	action_pressed.emit("update_addon")


func _handle_format_on_save_toggle(state: bool) -> void:
	setting_changed.emit("format_on_save", state)


func _handle_lint_on_save_toggle(state: bool) -> void:
	setting_changed.emit("lint_on_save", state)
