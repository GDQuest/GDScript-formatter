static func is_known_effect_kind(name: StringName) -> bool:
	return name in [
		&"harvest_multiplier", &"energy_bonus", &"free_return_trip",
		&"all_weather_roads", &"fertilizer_next_day_bloom", &"bloom_all_nodes",
		&"bloom_bonus_boost", &"restore_hunger", &"restore_sleepiness",
	]
