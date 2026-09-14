# #region / #endregion markers
# Region start and end comments stay next to the lines they enclose, but spacing
# is applied around the region based on the enclosed content.
#region Description
func test():
	pass
#endregion

#region AnotherRegion
func test2():
	pass
#endregion

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

## Test docstring
@warning_ignore("unused_parameter")
func nested(value):
	pass
#endregion
#endregion
#region D
func last():
	pass
#endregion
