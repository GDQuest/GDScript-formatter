#region Header
class_name RegionOrder extends Node
## Keeps class documentation after the header.
#endregion

var before_value = 0
signal before_changed

#region Outer
## Documents the exported value.
@export var outer_value = 1 # Stays with the value.
signal outer_changed
#region Inner
func _input(_event):
	pass
func _ready():
	pass
#endregion
var after_nested = 2
const AFTER_NESTED = 3
#endregion

#region Sibling
var sibling_value = 4
signal sibling_changed
#endregion

var after_value = 5
const AFTER = 6
