@tool
class_name Greeter
extends Window

signal action_pressed(action: String)

@onready var _addon_version_label: Label = $Container/Padding/Layout/Versions/Addon/Layout/Meta/Value
@onready var _formatter_version_label: Label = $Container/Padding/Layout/Versions/Formatter/Layout/Meta/Value
@onready var _heading_label: Label = $Container/Padding/Layout/Heading/Description

@onready var _docs_button: Button = $Container/Padding/Layout/Resources/Items/Docs/HBoxContainer/DocsButton
@onready var _issues_button: Button = $Container/Padding/Layout/Resources/Items/Issues/HBoxContainer/IssuesButton
@onready var _formatter_button: Button = $Container/Padding/Layout/Versions/Formatter/Layout/FormatterButton

var _addon_version: String = "0.0.0"
var _formatter_version: String = "0.0.0"


func _ready() -> void:
	_addon_version_label.text = _addon_version
	_formatter_version_label.text = _formatter_version
	
	close_requested.connect(_handle_close)
	
	_docs_button.button_down.connect(_handle_docs_press)
	_issues_button.button_down.connect(_handle_issues_press)
	_formatter_button.button_down.connect(_handle_formatter_press)
	
	_update_tagline()


func set_addon_version(version: String) -> void:
	_addon_version = version
	
	if is_node_ready():
		_addon_version_label.text = _addon_version


func set_formatter_version(version: String) -> void:
	_formatter_version = version
	
	if is_node_ready():
		_formatter_version_label.text = _formatter_version


func _update_tagline() -> void:
	var godot_version = Engine.get_version_info()
	var version_major = str(godot_version.major)
	
	_heading_label.text = _heading_label.text.replace("<MAJOR>", version_major)


func _handle_close() -> void:
	close_requested.disconnect(_handle_close)
	
	_docs_button.button_down.disconnect(_handle_docs_press)
	_issues_button.button_down.disconnect(_handle_issues_press)
	_formatter_button.button_down.disconnect(_handle_formatter_press)
	
	hide()


func _handle_docs_press() -> void:
	action_pressed.emit("help")


func _handle_issues_press() -> void:
	action_pressed.emit("report_issue")


func _handle_formatter_press() -> void:
	action_pressed.emit("install_update")
