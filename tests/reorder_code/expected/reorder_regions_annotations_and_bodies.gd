extends Node

@export
#region Exported
var exported_value = 1

signal must_follow_exported_value

var regular_value = 2
#endregion


func _ready():
	pass


func _input(_event):
	#region Function
	pass
	#endregion


class Nested:
	#region Members
	var value = 0
	signal changed


	#endregion
	func _input(_event):
		#region Body
		pass
		#endregion
