# This file tests long and complex expressions, especially parenthesized
# expressions, subscripts, and nested expressions. This directly tests a line
# length calculation for these expressions as there are just over 100 characters
# (counting tab indents as = 4 spaces)
func test_expressions():
	stored_direction_with_a_long_name = (
		transform.basis * Vector3(input_dir.x, 0, input_dir.y)
	).normalized()
	stored_direction_with_a_long_name += (
		transform.basis * Vector3(input_dir.x, 0, input_dir.y)
	)[index]
	return (
		first_direction_with_a_long_name + second_direction_with_a_long_name
	)[index].normalized()

	consume(
		(first_direction_with_a_long_name + second_direction_with_a_long_name)[index].normalized(),
		other_value,
	)
	var directions = [
		(first_direction_with_a_long_name + second_direction_with_a_long_name)[index].normalized(),
		other_value,
	]
	var direction_by_name = {
		"forward": (
			first_direction_with_a_long_name + second_direction_with_a_long_name
		)[index].normalized()
	}
	if (
		first_direction_with_a_long_name + second_direction_with_a_long_name
	)[selected_index].is_normalized():
		pass

	var result = (
		value
	).transform(
		first_argument_with_a_long_name,
		second_argument_with_a_long_name,
		third_argument_with_a_long_name,
	)
	var indexed = (
		value
	)[
		first_index_with_a_long_name + second_index_with_a_long_name
		+ third_index_with_an_extremely_long_name
	]
	var values = ([
		first_element_with_a_long_name,
		second_element_with_a_long_name,
		third_element_with_a_long_name,
	]).duplicate()
	var properties = ({
		"first": first_value_with_a_long_name,
		"second": second_value_with_a_long_name,
	}).duplicate()
	var doubled = (
		(first_direction_with_a_long_name + second_direction_with_a_long_name)
	)[index].normalized()
	var repeated_index = (
		first_direction_with_a_long_name + second_direction_with_a_long_name
	)[first_index][second_index]
	var selected = values[
		(
			first_index_with_a_long_name + second_index_with_a_long_name
			+ third_index_with_a_long_name
		)
	]
	var size = create_values(first_argument_with_a_long_name, second_argument_with_a_long_name)[
		index
	].size()
	var mapped = (values).map(
		func(value):
			return value * 2,
	)

	var transformed = (first_direction + second_direction) \
			.first_transformation() \
			.second_transformation() \
			.third_transformation()
	var explicit = (first_direction + second_direction) \
			.first_transformation() \
			.second_transformation() \
			.third_transformation()
