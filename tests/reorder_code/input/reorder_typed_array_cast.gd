static func make_defs() -> Variant:
	return [
		SomeType.new(&"aaa"),
		SomeType.new(&"bbb"),
		SomeType.new(&"ccc"),
		SomeType.new(&"ddd"),
	] as Array[SomeType]
