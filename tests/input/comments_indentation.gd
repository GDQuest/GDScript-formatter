func test() -> void:
	for x in range(10):
		if x != 5:
			continue
		# This comment should stay here
		print(x)

func test2():
	pass # This comment should stay here too

func test3() -> void:
	print("Test")
	# This comment should stay at the end of the function

# This comment should stay above the function
func test4() -> void:
	print("Test")