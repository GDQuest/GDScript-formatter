# A long parenthesized expression with a method call would not wrap as expected.
func _physics_process(delta: float) -> void:
	var direction_with_a_long_name: Vector3 = (
		transform.basis * Vector3(input_dir.x, 0, input_dir.y)
	).normalized()
	var basis_property_with_an_even_longer_name = (
		transform.basis * Vector3(input_dir.x, 0, input_dir.y)
	).x
	var indexed_value_with_a_long_name = (
		transform.basis * Vector3(input_dir.x, 0, input_dir.y)
	)[index]
	var indexed_direction_with_a_long_name = (
		transform.basis * Vector3(input_dir.x, 0, input_dir.y)
	)[index].normalized()
