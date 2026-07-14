@tool
@icon("res://icon.svg")
class_name TestClass
extends Node
## This is the class docstring

signal test_signal(value: int)

enum TestEnum {
	OPTION_A,
	OPTION_B,
	OPTION_C,
}
enum {
	UNNAMED_A,
	UNNAMED_B,
}

const TEST_CONSTANT = 42
const _PRIVATE_CONSTANT = "hidden"

static var static_variable = 100
static var _private_static_var = "static_private"

@export var exported_value: int = 10

var regular_variable: float = 3.14
var _private_regular: bool = true

@onready var onready_node: Node = get_child(0)
@onready var _private_onready: Timer = Timer.new()


static func _static_init():
	static_variable = 200


static func get_static_value() -> int:
	return static_variable


func _init():
	regular_variable = 2.71


func _enter_tree():
	pass


func _ready():
	pass


func _process(delta):
	pass


func _physics_process(delta):
	pass


func _exit_tree():
	pass


@rpc("authority", "call_remote", "reliable")
func public_method() -> int:
	return TEST_CONSTANT


func get_public_property() -> float:
	return regular_variable


func set_public_property(value: float):
	regular_variable = value


func _private_method() -> String:
	return _PRIVATE_CONSTANT


func _get_private_property() -> bool:
	return _private_regular


func _set_private_property(value: bool):
	_private_regular = value


class InnerClass:
	var inner_var: int = 5


	func inner_method():
		pass


class _PrivateInnerClass:
	var _inner_private: String = "inner"
