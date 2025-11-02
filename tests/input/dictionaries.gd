# The output has long lines wrapped to 100 characters and trailing commas.
var dialogue_items: Array[Dictionary] = [
	{"expression"   : expressions["regular"],"text": "I've been studying arrays and dictionaries lately.","character": bodies["sophia"]},
	{  "expression"  :  expressions[  "regular"] ,   "text": "Oh, nice. How has it been going?","character": bodies["pink"] },
]
# Single line dict should have a space after { and before }
var my_dictionary = {key = "value"}
# But only if it fits on one line
var my_dictionary_2 = {
	key = "value"}

var dict: Dictionary[int,int] = {}

const EXCEPTIONS: Dictionary[String, Dictionary] = {
	# UI scene
	"UI": {
		# Required to allow typing in the console
		"ConsoleInput": Control.FOCUS_ALL,
	},
}
