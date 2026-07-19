extends Control


func _gui_input(event: InputEvent) -> void:
	if event is InputEventKey:
		if (
			# Stop propagation of the ` key since it's used to open/close the console
			event.keycode == KEY_QUOTELEFT
			# Stop propagation of the escape key since we don't want to unfocus the input
			|| event.keycode == KEY_ESCAPE
			# Stop propagation of the up/down keys since they're used to navigate the command history
			|| event.keycode == KEY_UP
			|| event.keycode == KEY_DOWN
			# Stop propagation of the left, right, home, end, page up, and page down keys so that we can handle caret navigation ourselves
			|| event.keycode == KEY_LEFT
			|| event.keycode == KEY_RIGHT
			|| event.keycode == KEY_HOME
			|| event.keycode == KEY_END
			|| event.keycode == KEY_PAGEUP
			|| event.keycode == KEY_PAGEDOWN
			# Stop propagation of the tab key so we can use it for autocomplete
			|| event.keycode == KEY_TAB
		):
			accept_event()
