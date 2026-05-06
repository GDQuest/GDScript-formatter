# This copies code provided by @twilit-jack in
# https://github.com/GDQuest/GDScript-formatter/issues/224 (Thanks!)
# The issue compiled multiple cases combined here.
func _test() -> void:
	var my_very_long_condition := true
	var my_other_very_long_condition := false

	if (my_very_long_condition
			or my_other_very_long_condition):
		pass

	if (
			my_very_long_condition
			or my_other_very_long_condition
	):
		pass

	var party = [
		"Godot",
		"Godette",
		"Steve",
	]

	var character_dict = {
		"Name": "Bob",
		"Age": 27,
		"Job": "Mechanic",
	}

	enum Tile {
		BRICK,
		FLOOR,
		SPIKE,
		TELEPORT,
	}
