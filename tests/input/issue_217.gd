# Based on https://github.com/GDQuest/GDScript-formatter/issues/217
func _test() -> void:
	print("ABCDEF"
			.replace("A", "111111111111111111111111111111111111111111111111")
			.replace("B", "2")
			.replace("C", "3"))

	foo(
		"ABCDEF"
				.replace(
						"A",
						"111111111111111111111111111111111111111111111111"
				)
				.replace("B", "2")
				.replace("C", "3"),
		"second argument"
	)
