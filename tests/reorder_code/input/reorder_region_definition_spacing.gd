extends Node


func foo() -> void:
	print("foo")

#region A
func bar() -> void:
	print("bar")
#endregion
func after_region():
	pass
#region B (outer region)
#region C (nested inside B)

## A documented function.
@warning_ignore("unused_parameter")
func nested(value):
	pass
#endregion
#endregion
#region D
func last():
	pass
#endregion
