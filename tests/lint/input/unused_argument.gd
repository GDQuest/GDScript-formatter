#region Good

func good(used_arg, _unused_arg):
	print(used_arg)


func set_selection(node: Node) -> void:
	something.get_selection().add_node(node)
	something.edit_node(node)

#endregion

#region Bad

func bad(used_arg, unused_arg):
	print(used_arg)

#endregion
