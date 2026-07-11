# Property with setter and/or getter
# Tests both setter and getter syntaxes.
var  a  :  float  =  10.0   :
	set(  value   )  :    a = value
	get   :    return a
var second: int = 10

var b: float = 10.0:
	set(value):
		b = value
		print(123)
		pass
	get:
		print(123)
		pass
		return b

var prop: int = 0:
	set =set_prop,get= get_prop
