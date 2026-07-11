class_name TestClass
extends Node

# fmt: off
var vertices_preserve: PackedVector3Array = [
	Vector3(-1, 0, -1),
	Vector3( 1, 0, -1),
	Vector3( 1, 0,  1),
	Vector3(-1, 0,  1),
]
# fmt: on

var vertices_reformat: PackedVector3Array = [
	Vector3(-1, 0, -1),
	Vector3(1, 0, -1),
	Vector3(1, 0, 1),
	Vector3(-1, 0, 1),
]
