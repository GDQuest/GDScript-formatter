# Lambda spacing and body placement
var f = func():
	print(123)
var f2 = func():
	print(123)
	print(123)
var f3 = (func():
	print(123)
)
var f4 = (func():
	pass
)
var f5 = (func():
	print(123)
	print(123)
)
var f6 = (func():
	print(123)
	# comment in lambda
	print(456)
)


# The formatter was not idempotent, took two passes to stabilize the output.
# This regression tests that the formatter immediately settles on this result.
func issue_281() -> void:
	await (
		check(
			func() -> void:
				do_work(),
		).report("some quite long message that pushes this line well past the limit")
	)


# This ensures that we account for a quirk of the official GDScript parser:
# A lambda with surrounding parentheses needs the closing parenthesis to be
# either right at the end of the last line or to be indented like the last
# statement in the lambda function. Dedenting the closing parenthesis leads to a
# parse error.
func issue_287() -> void:
	var test := (
		(func(it: String) -> String:
			return it
		)
	)
	[].map(
		(func(it: String) -> String:
			return it
			)
	)
	CheatPanel.add_cheat(
		&'heal_hero',
		'Heal the hero',
		(func() -> void:
			health = max_health
			),
		self,
	)
