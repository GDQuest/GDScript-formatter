const bad_const = 42

const ignored_bad_const_1 = 100 # gdlint-ignore constant-name

# gdlint-ignore-next-line constant-name
const ignored_bad_const_2 = 100


class TestClass:
	func _private_method():
		pass


func bad():
	var obj = TestClass.new()
	obj._private_method()

	const badConstName = "this line is also very long but will NOT be ignored by the comment above this"


func ignored():
	var obj = TestClass.new()

	# gdlint-ignore-next-line private-access
	obj._private_method()

	obj._private_method() # gdlint-ignore private-access

	obj._private_method() # gdlint-ignore

	# gdlint-ignore-next-line
	obj._private_method()

	# gdlint-ignore-next-line constant-name,max-line-length
	const anotherBadConstName = "this line is also very long but it will be ignored by the comment above this"

	# gdlint-ignore-next-line
	const totallyIgnored = "this line is also very long but it will be ignored by the comment above this"
