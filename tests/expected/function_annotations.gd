# Annotations before functions
@abstract func foo():
	pass


@rpc
func foo():
	pass


@rpc("any_peer", "call_remote", "reliable")
func request_battle() -> void:
	pass


@warning_ignore("unused_parameter")
func with_warning(p: int) -> void:
	pass
