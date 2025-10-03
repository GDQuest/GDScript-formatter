var array = [1, 2, 3, 4, 5]

#region Good

func good():
	for i in array:
		print(i)

	for some_value in array:
		print(some_value)

#endregion

#region Bad

func bad():
	for A in array:
		print(A)

	for Something in array:
		print(Something)

	for _B in array:
		print(_B)

	for someValue in array:
		print(someValue)

	for _some_value in array:
		print(_some_value)

#endregion