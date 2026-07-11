# Blank lines between statements in functions
# We preserve up to one blank line between statements so that users can separate
# blocks of code but still automatically get them tidied up.
# TODO: See if users give feedback about this to ensure this is wanted. If not,
# work it out with multiple users.
func foo():
	print(123)

	print(123)

	# comment
	if true:


		while true:
			break


		for i in 10:
			continue


		if false:

			pass
		else:

			pass

		match "1":
			_:
				pass
