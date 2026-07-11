# Semicolons as statement separators
# Semicolons should be removed from the end of single statements. `; ;` should
# merge. Semicolons inside strings should be preserved.
var a = 10;
var b = "  ; ";


func foo():
	print(123);
	pass;


func bar():
	var c = 2;
	var d = 3;
	if c < d:
		print("c is less than d");
	if c > d:
		print("a is greater than b");


func test():
	print("A") ; ; print("B")

func a(): print("a");
