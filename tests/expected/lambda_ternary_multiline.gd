# Lambda inside a ternary expression
var my_callable: Callable = (
	(func():
		var my_very_long_var_definition = 1
	)
	if some_very_long_condition
	else some_very_long_function_name
)
