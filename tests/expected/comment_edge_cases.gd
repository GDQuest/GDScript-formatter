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
