extends Node

signal died

var armor = 0

#region Stats
## Emitted when health reaches zero.
signal health_changed(new_health)

enum States {
	IDLE,
	RUNNING,
	ATTACKING,
	DEAD,
}

const MAX_HEALTH = 100

@export var max_health := MAX_HEALTH

var states := States.IDLE

@onready var health_bar: ProgressBar = %HealthBar


func _ready():
	pass


func _process(delta):
	pass


@rpc("any_peer")
func emit_health_changed(new_health):
	health_changed.emit(new_health)


#region Flags
var is_alive := true
#endregion

var health := max_health


## Applies damage and reports the new value.
func apply_damage(amount):
	health -= amount
	emit_health_changed(health)
#endregion


#region Comments only
# This region has no declarations to reorder.
#endregion


func after_all_regions():
	pass
