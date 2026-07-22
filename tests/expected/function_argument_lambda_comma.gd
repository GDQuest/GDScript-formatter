# Lambda as a function call argument with trailing comma
# This regression test verifies the lambda trailing comma gets preserved after
# formatting.
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


# An unparenthesized multiline lambda argument needs a trailing comma so the
# official parser accepts the closing call parenthesis.
func test_single_lambda_argument():
	var test: String = ", ".join(
		[].map(
			func(it: String) -> String:
				return it,
		)
	)
