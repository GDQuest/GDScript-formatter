# Comment placement and preservation across code
# Comments should be preserved as-is after formatting. This file tries to
# capture different kinds of comments with some that should attach to a
# definition before or after them.
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


func test_function():
	var a = "test"

	# This comment should stay inside of the function body
	print(a)


func test() -> void:
	for x in range(10):
		if x != 5:
			continue
		# This comment should stay here
		print(x)


func test2():
	pass # This comment should stay here too


func test3() -> void:
	print("Test")
	# This comment should stay at the end of the function


# This comment should stay above the function
func test4() -> void:
	print("Test")

@export_group("my_group") # annotation comment

var prop = 10:
	# var comment
	set(value): # set comment
		prop = value
	get: # get comment
		return prop

enum Foo {
	A, # Comment
	B, # Comment
	C,
}


class InnerClass: # class comment
	pass


func _init(): # constructor comment
	var lua_dict = {
		# Comment
		a = 0, # Comment
		# Comment
		b = 1, # Comment
		# Comment
	}

	var arr = [
		1, # Comment
		2, # Comment
		# Comment
		2, # Comment
		# Comment 2
		2, # Comment
		3, # Comment
	]
	pass


func foo(): # func comment
	if true: # if comment
		pass
	elif false: # elif comment
		pass
	else: # else comment
		pass

	match 0: # match comment
		1: # case comment
			pass
		_: # default comment
			pass

	for i in 10: # for comment
		pass

	while false: # while comment
		pass

	var lam = func(): # lambda comment
		pass

	bar(
		a, # function call inline comment
		b,
	)

	return # function trailing comment at end


func comment_after_parameter(
	a,
	b, # should stay attached to parameter
	c,
):
	pass
