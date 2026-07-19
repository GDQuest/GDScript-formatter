extends Node

func accelerate_mob() -> void:
	mob.velocity = mob.velocity.move_toward(desired_velocity, velocity_distance * acceleration_factor * delta)


func test() -> void:
	# Long argument in the middle of the chain.
	result.start().transform("0123456789012345678901234567890123456789012345678901234567890123456789").finish()
