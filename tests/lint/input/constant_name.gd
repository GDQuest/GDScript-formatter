#region Good

const GOOD_NAME = 42
const _PRIVATE_GOOD_NAME = 100

# preload can be CONSTANT_CASE or PascalCase (i.e. a ClassName)
const GOOD_LOAD = preload("res://path/to/resource.tscn")
const ClassNameLoad = preload("res://path/to/resource2.tscn")

#endregion

#region Bad

const bad_name = 42
const _private_bad_name = 43
const AnotherBadName = 44

const bad_preload = preload("res://path/to/resource3.tscn")

# this is technically invalid GDScript since `load` can't be assigned to a constant
const BAD_LOAD = load("res://path/to/resource4.tscn")

#endregion