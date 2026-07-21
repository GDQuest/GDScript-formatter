# Generic type parameters must never break across lines, even when they would go
# past max line length. Breaking brackets like Dictionary[ String, String] would
# produce invalid GDScript.
func return_nested_generic() -> Dictionary[String, Array[int]]:
	return { "a": [1, 2, 3], "b": [4, 5, 6] }


func typed_variable() -> void:
	var d: Dictionary[String, int] = {}
	print(d)


func parameterized() -> void:
	pass
