extends Node


func connect_overlay() -> void:
	_view_overlay.gui_input.connect(
		func(event: InputEvent) -> void:
			_panel_gui_input(event)
			_view_overlay.mouse_default_cursor_shape = _bubble_container.mouse_default_cursor_shape,
	)


static var _erase_arg_name_transformer := func(f: Dictionary) -> Dictionary:
	f.args.map(
		func(arg: Dictionary) -> Dictionary:
			arg.erase("name")
			return arg,
	)
	return f
