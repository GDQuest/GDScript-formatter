#region Good

func good(value):
	if value < 0:
		print("Negative value")
		return -1
	
	if value > 100:
		print("Too large")
		return 100
	
	return value

#endregion

#region Bad

func bad(value):
	if value < 0:
		print("Negative value")
		return -1
	elif value > 100:
		print("Too large")
		return 100
	else:
		return value

#endregion