# Blank lines inside lambda bodies passed as call arguments
# We want to ensure we preserve up to 1 user blank lines inside a lambda body.
func f9():
	connect(
		func():
			print("THIS LINE IS CORRECT")


			print("THIS LINE IS INCORRECT")
	)
