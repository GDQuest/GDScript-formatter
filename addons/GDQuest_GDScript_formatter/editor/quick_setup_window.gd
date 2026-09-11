## Quick setup wizard window that we display when the user first enables the
## plugin.
@tool
class_name QuickSetupWindow
extends Window

enum ButtonActions {
	HELP,
	REPORT_ISSUE,
	INSTALL_UPDATE,
	UNINSTALL,
	UPDATE_ADDON,
	OPEN_WEBSITE,
}
enum Settings {
	FORMAT_ON_SAVE,
	LINT_ON_SAVE,
}

signal button_pressed(action: ButtonActions)
signal setting_change_requested(setting: Settings, value: Variant)

@onready var _addon_version_label: Label = %AddonVersionLabel
@onready var _formatter_version_label: Label = %FormatterVersionLabel

@onready var _docs_button: Button = %DocsButton
@onready var _issues_button: Button = %IssuesButton
@onready var _formatter_install_button: Button = %FormatterInstallButton
@onready var _formatter_uninstall_button: Button = %FormatterUninstallButton
@onready var _addon_install_button: Button = %AddonInstallButton
@onready var _format_on_save_toggle: CheckButton = %FormatOnSaveToggle
@onready var _lint_on_save_toggle: CheckButton = %LintOnSaveToggle
@onready var _quick_settings_icon: TextureRect = %QuickSettingsIcon
@onready var _resources_icon: TextureRect = %ResourcesIcon
@onready var _logo: TextureButton = %Logo

var _addon_version := "-"
var _formatter_version := "-"
var _setting_states := { }


func _ready() -> void:
	var is_editing_scene := (Engine.is_editor_hint() and get_tree().edited_scene_root == self)
	if not is_editing_scene:
		var editor_scale := EditorInterface.get_editor_scale()
		var button_icon_size := roundi(16.0 * editor_scale)
		var section_icon_size := Vector2.ONE * 18.0 * editor_scale
		_logo.custom_minimum_size = Vector2(96.0, 29.6) * editor_scale
		for button: Button in [
			_addon_install_button,
			_formatter_install_button,
			_formatter_uninstall_button,
			_docs_button,
			_issues_button,
		]:
			button.add_theme_constant_override("icon_max_width", button_icon_size)
		_quick_settings_icon.custom_maximum_size = section_icon_size
		_resources_icon.custom_maximum_size = section_icon_size
	_logo.pressed.connect(
		func() -> void:
			button_pressed.emit(ButtonActions.OPEN_WEBSITE),
	)

	_addon_version_label.text = _addon_version
	_formatter_version_label.text = _formatter_version

	close_requested.connect(
		func() -> void:
			hide(),
	)
	_docs_button.button_down.connect(
		func() -> void:
			button_pressed.emit(ButtonActions.HELP),
	)
	_issues_button.button_down.connect(
		func() -> void:
			button_pressed.emit(ButtonActions.REPORT_ISSUE),
	)
	_formatter_install_button.button_down.connect(
		func() -> void:
			button_pressed.emit(ButtonActions.INSTALL_UPDATE),
	)
	_formatter_uninstall_button.button_down.connect(
		func() -> void:
			button_pressed.emit(ButtonActions.UNINSTALL),
	)
	_addon_install_button.button_down.connect(
		func() -> void:
			button_pressed.emit(ButtonActions.UPDATE_ADDON),
	)
	_format_on_save_toggle.toggled.connect(
		func(state: bool) -> void:
			setting_change_requested.emit(Settings.FORMAT_ON_SAVE, state),
	)
	_lint_on_save_toggle.toggled.connect(
		func(state: bool) -> void:
			setting_change_requested.emit(Settings.LINT_ON_SAVE, state),
	)

	_update_setting_ui()


func set_addon_version(version: String) -> void:
	_addon_version = version
	if is_node_ready():
		_addon_version_label.text = _addon_version


func set_formatter_version(version: String) -> void:
	_formatter_version = version
	_formatter_uninstall_button.disabled = _formatter_version == "-"
	if is_node_ready():
		_formatter_version_label.text = _formatter_version


func set_setting_state(setting: Settings, value: Variant) -> void:
	_setting_states.set(setting, value)
	if is_node_ready():
		_update_setting_ui()


func _update_setting_ui() -> void:
	if _setting_states.has(Settings.FORMAT_ON_SAVE):
		var state = _setting_states.get(Settings.FORMAT_ON_SAVE)
		_format_on_save_toggle.button_pressed = state == true

	if _setting_states.has(Settings.LINT_ON_SAVE):
		var state = _setting_states.get(Settings.LINT_ON_SAVE)
		_lint_on_save_toggle.button_pressed = state == true
