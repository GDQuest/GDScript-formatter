# Function call arguments wrapped across lines
# Tests various multiline function call styles: opening paren on its own line, closing paren on its own line, and mixed styles. Continuation lines get +2 indent with a trailing comma. Also covers method chaining with vertical alignment at dots.
func test():
	print(
			"Testing a long enough chain",
			"of arguments to produce a multiline",
			"print function call",
	)

	print(
			"Testing a long enough chain",
			"of arguments to produce a multiline",
			"print function call",
	)

	print(
			"Testing a long enough chain",
			"of arguments to produce a multiline",
			"print function call",
	)

	print(
			"Testing a long enough chain",
			"of arguments to produce a multiline",
			"print function call",
	)

	print(
			"Testing a long enough chain",
			"of arguments to produce a multiline",
			"print function call",
	)


# Verify the automated line wrapping of chained function calls is as desired.
func _test() -> void:
	print(
			"ABCDEF"
			.replace("A", "111111111111111111111111111111111111111111111111")
			.replace("B", "2")
			.replace("C", "3")
	)

	foo(
			"ABCDEF"
			.replace("A", "111111111111111111111111111111111111111111111111")
			.replace("B", "2")
			.replace("C", "3"),
			"second argument",
	)
