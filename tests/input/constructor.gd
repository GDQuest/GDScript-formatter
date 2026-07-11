# _init constructor and surrounding blank lines
# In the parser _init() is a special case so we ensure that blank lines get
# inserted correctly like any other function.
var a = 1
func _init():
	print(123)
func foo():
	pass
