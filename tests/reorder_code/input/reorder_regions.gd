extends Node

signal died

var armor = 0

#region Stats

var states := States.IDLE

func _process(delta):
	pass

## Emitted when health reaches zero.
signal health_changed(new_health)

const MAX_HEALTH = 100

@rpc("any_peer")
func emit_health_changed(new_health):
	health_changed.emit(new_health)

@export var max_health := MAX_HEALTH

func _ready():
	pass

enum States {IDLE, RUNNING, ATTACKING, DEAD}

@onready var health_bar: ProgressBar = %HealthBar

#region Flags
var is_alive := true
#endregion

## Applies damage and reports the new value.
func apply_damage(amount):
	health -= amount
	emit_health_changed(health)

var health := max_health

#endregion

#region Comments only
# This region has no declarations to reorder.
#endregion

func after_all_regions():
	pass
