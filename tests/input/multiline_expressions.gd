# Multi-line expressions
# Tests things like ternary in parentheses, nested arrays, bitflags, and long
# expressions that should wrap over multiple lines
# TODO: work on long ternary expressions or boolean expressions.
func foo():
	var angle_degrees = 135
	var quadrant = (
				"northeast"    if     angle_degrees <= 90

				else "southeast" if angle_degrees <= 180
				else "southwest" if angle_degrees <= 270
				else "northwest"
		)

	var quadrant_newlines = (
		"northeast"
		if angle_degrees <= 90
		else "southeast"

		if angle_degrees <= 180
		else "southwest"

		if angle_degrees <= 270
		else "northwest"
	)

	# When the ternary expression does not already have surrounding parentheses
	# or continuation lines and wraps, we need to surround it with parentheses
	# for GDScript to parse it.
	var pitch_input = Input.get_axis("ui_up", "ui_down") if invert_pitch else Input.get_axis(
		"ui_down",
		"ui_up"
	)

	var position = Vector2(250, 350)
	if position.x > 200 and position.x < 400 and position.y > 300 and position.y < 400 and position.z > 0 and position.z < 100:
		pass


	var a =    (

		    1 + 2

	)

	var sum =    (1 +
2
)

	var bitflags = (
				0x0b
			| 0xa0
		)


# Multi-line if conditions with `or`, parenthesized condition across lines. Code contributed by @twilit-jack.
func _test() -> void:
	var my_very_very_long_condition_long_enough_to_wrap := true
	var my_other_very_very_long_condition_long_enough_to_wrap := false

	# TODO: think if this should fall under the continuation line guideline.
	if (my_very_very_long_condition_long_enough_to_wrap
				or my_other_very_very_long_condition_long_enough_to_wrap):
		pass

	if (
				my_very_very_long_condition_long_enough_to_wrap
				or my_other_very_very_long_condition_long_enough_to_wrap
	):
		pass
