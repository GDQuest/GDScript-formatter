var is_valid: bool = some_condition \
		and another_condition


func _handles(resource: Resource) -> bool:
	return resource is NoiseTexture2D \
			or resource is GradientTexture1D


func _process(delta: float) -> void:
	if is_on_floor() \
			and not is_jumping:
		apply_gravity(delta)


func _calculate() -> float:
	var x: float = some_long_value \
			+ another_value
	return x


func _ready() -> void:
	node.set_position(Vector2.ZERO) \
			.rotated(PI)


# string with \ must not change indentation
func _get_message() -> String:
	var greeting: String = "\
	Hello world"
	return greeting
