@tool
class_name Greeter
extends Window

@onready var _addon_version_label_value: Label = $Panel/Padding/Layout/Versions/Addon/Layout/Meta/Value
@onready var _formatter_version_value: Label = $Panel/Padding/Layout/Versions/Formatter/Layout/Meta/Value

var _addon_version: String = "0.0.0"
var _formatter_version: String = "0.0.0"

func _init() -> void:
	close_requested.connect(_handle_close)


func _ready() -> void:
	_addon_version_label_value.text = _addon_version
	_formatter_version_value.text = _formatter_version


func set_addon_version(version: String) -> void:
	_addon_version = version
	
	if is_node_ready():
		_addon_version_label_value.text = _addon_version


func set_formatter_version(version: String) -> void:
	_formatter_version = version
	
	if is_node_ready():
		_formatter_version_value.text = _formatter_version


func _handle_close() -> void:
	hide()
