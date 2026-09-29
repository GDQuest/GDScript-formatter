func select_room() -> void:
	var usedRoomSelectorIndex := availableRoomSelectors.find_custom(
		(func(iter: RoomSelector, expect: RoomLayoutInfo) -> bool:
			return iter.layoutInfo == expect).bind(selectedRoom)
	)
