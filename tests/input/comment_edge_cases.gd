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
