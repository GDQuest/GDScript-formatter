extends Node

var    ugly_before   =   1


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
	print(   "hi"   )

# fmt: off
var y    =    2

func bar2():
	var a    =    1
	# fmt: on
	var b    =    3
	print(   "after"   )


func foo3():
	var x    =    1
	# fmt: off
	var y    =    2
	if true:
		pass

func bar3():
	print(   "hi"   )
