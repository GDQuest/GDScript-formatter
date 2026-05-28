func jump(force: float) -> void:
	self.velocity.y = force
	is_shooting = false


func shoot() -> void:
	var pos: Vector2 = muzzle_right.global_position
	var direction: Vector2 = Vector2.RIGHT
	if is_facing_left:
		pos = muzzle_left.global_position
		direction = Vector2.LEFT

	var lazer: Projectile = Instancer.instance_scene_to_level(Instancer.laster_scene, pos) as Projectile
	if lazer != null:
		lazer.launch(direction, lazer_speed)

	is_shooting = false
	var yozora: bool.
