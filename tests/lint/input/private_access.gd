#region Good
class BaseClass:
	var _private_var = 42


	func _private_method():
		pass


	func public_method():
		self._private_method()
		_private_method()


class DerivedClass extends BaseClass:
	func another_method():
		# calling super private method is allowed
		super._private_method()

		_private_var = 100 # accessing super private var is allowed


func good():
	var obj = BaseClass.new()
	obj.public_method()

#endregion

#region Bad

func bad():
	var obj = BaseClass.new()
	obj._private_method()

	obj._private_var = 100

#endregion
