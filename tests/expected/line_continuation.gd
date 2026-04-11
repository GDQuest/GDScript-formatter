func _handles(resource: Resource) -> bool:
	return resource is NoiseTexture2D \
			or resource is GradientTexture1D \
			or resource is GradientTexture2D
