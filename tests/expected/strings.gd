# String variants
# Tests all variants of "strings": with single, double, triple quotes, node
# paths, and StringName
extends Node

var simple_string = "Hello, World!"
var string_with_quotes = 'This string uses single quotes'
var multiline_string = """This is a
multiline string that
spans multiple lines
"""
var string_with_escapes = "Line 1\nLine 2\tTabbed"
var empty_string = ""
var string_name = &"Name"
var node_path = ^"Node/Path"
