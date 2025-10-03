#region Good

func good_function(used_arg, _unused_arg):
	print(used_arg)

func good_typed_function(used_arg: int, _unused_arg: String):
	print(used_arg)

#endregion

#region Bad

func bad_function(badArg, BadArg, _BadUnusedArg):
	print(badArg, BadArg)

func bad_typed_function(badArg: int, BadArg: Array, _BadUnusedArg: String):
	print(badArg, BadArg)

#endregion