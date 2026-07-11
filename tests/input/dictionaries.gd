# Dictionary spacing, wrapping, and nested structures
# Tests single-line dict spacing, multi-line dicts, inline dicts inside arrays,
# nested dicts with comments, and type hints.
var dialogue_items: Array[Dictionary] = [
	{"expression"   : expressions["regular"],"text": "I've been studying arrays and dictionaries lately.","character": bodies["sophia"]},
	{  "expression"  :  expressions[  "regular"] ,   "text": "Oh, nice. How has it been going?","character": bodies["pink"] },
]
# Single line dict should have a space after { and before }
var my_dictionary = {key = "value"}

var dict: Dictionary[int,int] = {}

const EXCEPTIONS: Dictionary[String, Dictionary] = {
	# UI scene
	"UI": {
		# Required to allow typing in the console
		"ConsoleInput": Control.FOCUS_ALL,
	},
}
