# first line comment
@tool
class_name Aaa
extends Node

# after extends comment

# before statement comment
var a = 10 # inline comment
# after statement comment

var b = 10
# after statement comment

func do_thing() -> void:
	if true:
		# this is a comment inside of the function block
		pass
		
		# this is a comment at the end of the function block
		# it should stay inside of this function block


func do_another_thing() -> void:
	pass

	# likewise, this comment should stay inside of this function block