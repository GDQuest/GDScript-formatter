#region Good

var assigned = "hello"

func good():
	var also_assigned = "world"
	print(assigned)
	print(also_assigned)

	var inner_function = (func():
		var inner_assigned = 42
		print(inner_assigned))

	inner_function.call()

#endregion

#region Bad

"I am not assigned"

func bad():
	"I am not assigned"
	42
	true
	false
	null

#endregion