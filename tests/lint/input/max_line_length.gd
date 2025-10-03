#region Good

var short_and_sweet = "hello"

#endregion

#region Bad

var really_long_variable_name = "This is a really long string that exceeds the line length of 100 that is configured by default"


func bad():
	var array = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24]

	for i in array:
		for j in array:
			for k in array:
				for l in array:
					for m in array:
						print("Here is another very long line, this one deeply nested inside of ton of loops")
#endregion
