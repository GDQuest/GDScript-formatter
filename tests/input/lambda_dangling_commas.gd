# Lambda as argument in function call and in collections
tween.tween_method(func(x: float) -> void:
		if x <= 1.0:
			print(x)
	,
	0.0,
	1.0,
	0.37
)

var g: Array[Callable] = [
	func():
		pass
	,
	func():
		pass
	,
]
