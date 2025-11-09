func test():
	hurt_tween.tween_method(
		func(weight):
			var hue = interpolate_hue(current_hue, target_hue, weight)
			# The tailing comma here should stay there as it completes the
			# lambda argument
			starnest.set_instance_shader_parameter("hue_shift", hue),
		0.0,
		1.0,
		2.0,
	)
