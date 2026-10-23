#region Header
class_name RegionOrder
extends Node
## Keeps class documentation after the header.
#endregion

signal before_changed

var before_value = 0

#region Outer
signal outer_changed

## Documents the exported value.
@export var outer_value = 1 # Stays with the value.


#region Inner
func _ready():
	pass


func _input(_event):
	pass
#endregion


const AFTER_NESTED = 3

var after_nested = 2
#endregion

#region Sibling
signal sibling_changed

var sibling_value = 4
#endregion

const AFTER = 6

var after_value = 5
