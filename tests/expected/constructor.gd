# _init constructor and surrounding blank lines
# In the parser _init() is a special case so we ensure that blank lines get
# inserted correctly like any other function.
var a = 1


func _init():
	print(123)


func foo():
	pass


# Testing a long definition line to ensure it wraps. Code from #370
func _init(
	node_name: String = "SkidMarks",
	shader_path: String = SHADER,
	fade_s: float = FADE_S,
) -> void:
	name = node_name
