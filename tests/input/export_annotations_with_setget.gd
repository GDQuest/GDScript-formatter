@export_range(0., 500., .1, "or_greater", "suffix:m")
var max_distance_test: float = 200.:
	set(value):
		max_distance_test = value

@export_range(0., 500., .1, "or_greater", "suffix:m")
var getter_distance: float = 200.:
	get:
		return getter_distance

@export_range(0., 500., .1, "or_greater", "suffix:m")
var both_distance: float = 200.:
	set(value):
		both_distance = value
	get:
		return both_distance

@export_enum("VeryLongFirstChoice", "VeryLongSecondChoice", "VeryLongThirdChoice", "VeryLongFourthChoice")
var selected_option: int = 0

@export_group("Orbit limits")
@export_range(0., 50., .1, "or_greater", "suffix:m")
var grouped_distance: float = 2.:
	set(value):
		grouped_distance = value
