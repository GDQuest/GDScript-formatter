# Annotations on variables
@export var max_health := 5
@onready var health := max_health
@onready var health := max_health
@export_range(10.0, 200.0) var jump_height := 50.0
@export_range(0.1, 1.5) var jump_time_to_peak := 0.37

@export_group("my group")
@export var v = 1
