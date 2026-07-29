# Enum spacing and wrapping
# Enums are always vertical, with one indent and trailing commas.
enum {
	A,
	B,
	C,
}

enum Named {
	A,
	B,
	C,
}

enum ThirdEnum {
	Aaaaaaaaaaaaaaaaaaaaaa,
	Bbbbbbbbbbbbbbbbbbbbbb,
	Cccccccccccccccccccccc,
	Dddddd,
	Eeeeee,
	Ffffff,
	Gggggg,
	Hhhhhh,
}


func foo():
	pass


enum Test {
	A,
	B,
}

enum Single {
	ONLY,
}

enum Event {
	NONE,
	FINISHED,

	PLAYER_EXITED_LINE_OF_SIGHT,
	PLAYER_ENTERED_LINE_OF_SIGHT,
	PLAYER_ENTERED_ATTACK_RANGE,

	TOOK_DAMAGE,
	HEALTH_DEPLETED,
	PLAYER_DIED,
}
