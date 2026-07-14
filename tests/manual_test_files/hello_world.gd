# Tests that the editorconfig file works as intended when running the formatter
# over a directory.
extends Node


func _ready():
	print("Hello, world!")

	print("'Hello' again!")

	var test = """
	Multiline "string" here
	"""
