extends Node

var early_variable: int = 1

signal my_signal(value: int)

const MY_CONST = 42

@onready var node: Node = get_child(0)

func _ready():
	pass

@export var exported: bool = false
