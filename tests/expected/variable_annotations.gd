# Annotations on variables
@export var max_health := 5
@onready var health := max_health
@onready var health := max_health
@export_range(10.0, 200.0) var jump_height := 50.0
@export_range(0.1, 1.5) var jump_time_to_peak := 0.37
@export_custom(
	PROPERTY_HINT_TYPE_STRING,
	"%d/%d:Zero,One,Two" % [TYPE_INT, PROPERTY_HINT_ENUM],
) var test_export_custom: Array

@export_group("my group")
@export var v = 1

@warning_ignore("unsafe_property_access")
var url: String = (_js_window.location.hash.trim_prefix("#").trim_prefix("/"))

# @export and @onready annotations should merge with the variable declaration
# even when the expression wraps over multiple lines.
@export var exported_url: String = (
	_js_window.location.hash.trim_prefix("#").trim_prefix("/")
	+ _js_window.location.hash.trim_prefix("#").trim_prefix("/")
)

@onready var onready_url: String = (
	_js_window.location.hash.trim_prefix("#").trim_prefix("/")
	+ _js_window.location.hash.trim_prefix("#").trim_prefix("/")
)


class AnnotationsInClassBody:
	@warning_ignore("unsafe_property_access")
	var url: String = (_js_window.location.hash.trim_prefix("#").trim_prefix("/"))

	@export var exported_url: String = (
		_js_window.location.hash.trim_prefix("#").trim_prefix("/")
		+ _js_window.location.hash.trim_prefix("#").trim_prefix("/")
	)
