extends Node

var ugly_before = 1

# fmt: off
var   m  =  Vector3( 1 , 0 , -1 )
var    n =    2
# fmt: on


func after():
	if true:
		# fmt: off
		var    x =   1
		var y=2
		# fmt: on
		print(x)


func foo2():
	print("hi")


# fmt: off
var y    =    2

func bar2():
	var a    =    1
	# fmt: on
	var b = 3
	print("after")


func long_definition_with_disabled_body(
	first_parameter: int,
	second_parameter: int,
	third_parameter: int,
	fourth_parameter: int,
) -> void:
	var before = 1
	# fmt: off
	var inside  =  [ 1 ,  2 ]
	# fmt: on
	var after = 2


func nested_disabled_region(
	first_parameter: int,
	second_parameter: int,
	third_parameter: int,
	fourth_parameter: int,
) -> void:
	if true:
		var before = 1
		# fmt: off
		var inside  =  [ 1 ,  2 ]
		# fmt: on
		var after = 2
	var after_if = 3


func disabled_parameters(
	first: int, # fmt: off
second  :  int, third:int, # fmt: on
	fourth: int,
) -> void:
	var array = [
		0, # fmt: off
	1 ,  2 , # fmt: on
		3,
	]
	var dictionary = {
		"before": 0, # fmt: off
	"inside"  :  1 , # fmt: on
		"after": 2,
	}
	call_something(
		0, # fmt: off
	1 ,  2 , # fmt: on
		3,
	)


func foo3():
	var x = 1
	# fmt: off
	var y    =    2
	if true:
		pass

func bar3():
	print(   "hi"   )
