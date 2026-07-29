class_name StatsData
extends Resource

@export_category("Survival")
@export_range(1, 999) var max_health: int

@export_category("Movement")
@export var can_move: bool
@export var movement_acceleration: float
@export_range(0.01, 10.0) var movement_speed: float

@export_category("Perception")
@export_group("Vision", "vision")
@export var can_see: bool
@export_subgroup("Details")
@export_flags_2d_physics var vision_collision_layer: int
