var f = func(): print(123)
var f2 = func():
	print(123)
	print(123)
var f3 = (func(): print(123))
var f4 = (func(): pass)
var f5 = (func():
	print(123)
	print(123)
)
var f6 = (func():
	print(123)
	# comment in lambda
	print(456)
)
