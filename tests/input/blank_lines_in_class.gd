# Blank line rules inside scripts
# This ensures that we have two blank lines by default around top level function
# and inner class declarations. Comments get attached to functions or inner
# classes and move with them.
#
# The gap between functions is configurable, this only validates the default
# (following the official GDScript styleguide).
var a

# case 1
func test2() -> void:
	pass

# Multiline docstring
# another line
func test2() -> void:
	pass

var a # case 2

func test2() -> void:
	pass

var a

func test2() -> void:
	pass

const a = 10

func test2() -> void:
	pass
var x = 10

class CheckSameCasesInsideNestedClass:
	var a

	# case 1
	func test2() -> void:
		pass

	var a # case 2

	func test2() -> void:
		pass

	var a

	func test2() -> void:
		pass

	const a = 10

	func test2() -> void:
		pass

	class DoubleNestedCaseWithInlineComments:
		var a # case 2

		func test2() -> void:
			pass

var a # This is a comment

# comment

# Ready
func test():
	pass

# Ready
func test():
	pass

var a # This is a comment

# Ready
func test():
	pass


var a # This is a comment
# Ready
func test():
	pass

var a # This is a comment
func test():
	pass

var a # This is a comment
# Two line
# documentation
func test():
	pass
@rpc
func test_rpc() -> void:
	pass
