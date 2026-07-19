extends Node

func test() -> void:
	mob.velocity = mob.this_is_an_unusually_long_property_name_that_makes_the_call_prefix_exceed_the_line_limit.move_toward(desired_velocity, velocity_distance * acceleration_factor * delta * extra_acceleration_multiplier)
