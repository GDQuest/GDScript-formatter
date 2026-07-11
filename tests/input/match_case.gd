# Match statements
# Should test all match statement variants including single and multiple
# patterns, binds, when guards, conditional expressions patterns, and
# annotations on match cases.
func foo():
	match   "1"  :
		"1"  :
			pass
		_   :
			pass
	match   "1"  :
		"1",    "2","3"  :
			pass
		_   :
			pass
	match 0:
		# Warnings and comments should be preserved
		@warning_ignore("unreachable_pattern")
		0 when true:
			print("true")
	match 5:
		# Conditional expression should be supported
		0 if true else 2:
			pass

	match type:
		Type.LEFT: animation.play("left")
		Type.RIGHT: animation.play("right")
		_: animation.play("right")
