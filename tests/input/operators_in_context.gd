# Binary/bitwise/unary operators inside containers and parentheses
func foo():
	var a = [1 + 2,2]
	a = [1+2]
	a = [1+2+3]
	a = (   1+2 )
	a = ( 1    | 2|3    )
	a = ( 1    & 2&3    )
	a = ( 1    ^ 2^3    )
	a = ( 1    << 2<<3    )
	a = ( 8    >> 2>>1    )
	a = false&&    true
	a = ~ 1
	a = + 1
	a = - 1
	if not valid:
		pass
	if ! valid:
		pass
