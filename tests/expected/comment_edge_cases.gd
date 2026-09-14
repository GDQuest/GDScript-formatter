# Tests that the line spacing between comments is preserved after formatting.
func test():
	# first comment

	# second comment
	pass


func test2():
	if true:
		pass
		# first comment
	# second comment
	elif false:
		pass


func test3():
	# This test ensures that we preserve up to one empty line between
	# conditional blocks.
	if true:
		pass

	elif false:
		pass

	else:
		pass


func test_inline_comment_preserves_blank_line():
	some_call() # inline comment

	another_call()


func test_inline_comment_no_blank_line():
	some_call() # inline comment
	another_call()


func test_inline_comment_multiple_blank_lines_collapse():
	some_call() # inline comment

	another_call()
