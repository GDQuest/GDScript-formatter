#region Good

var x = 5
var y = 10


func good():
	if x == y:
		pass

#endregion

#region Bad

func bad():
	if x == x:
		pass

#endregion
