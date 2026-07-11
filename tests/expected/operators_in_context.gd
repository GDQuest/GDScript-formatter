# Binary/bitwise/unary operators inside containers and parentheses
func foo():
	var a = [1 + 2, 2]
	a = [1 + 2]
	a = [1 + 2 + 3]
	a = (1 + 2)
	a = (1 | 2 | 3)
	a = false && true
	a = ~1
