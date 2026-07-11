# Trailing comma placement in containers
# Containers should have a trailing comma only when elements are wrapped on
# multiple lines it. preload() should have no trailing comma because the
# GDScript grammar does not accept a trailing comma in preload calls.
var a = [
	1,
	2,
	3,
]

var aa = [1, 2, 3]

var b = [
	1,
	2,
	3, # comment
]

var c = {
	"a": 1,
	"b": 2,
	"c": 3, # comment
}

var d = {
	"a": 1,
	"b": 2,
	"c": 3,
}

var dd = { "a": 1, "b": 2, "c": 3 }

enum Foo {
	A,
	B,
	C,
}

enum Foo2 {
	A,
	B,
	C, # comment
}

enum Foo3 { A, B, C }


func foo(
		a,
		b,
):
	pass


func bar(
		a,
		b, # comment
):
	pass


func f():
	foo(
			1,
			2,
	)


func test(a: int, b: int):
	pass


func test():
	print("test", "test")


# Preload does not support trailing commas, multiline formatting should not add it
const MAIN_MUSIC: AudioStream = preload(
		"res://assets/audio/a_very_long_filename_that_honestly_could_be_a_lot_shorter.mp3"
)


# Containers with practical data from issue #224 (contributed by @twilit-jack)
var party = [
	"Godot",
	"Godette",
	"Steve",
]

var character_dict = {
	"Name": "Bob",
	"Age": 27,
	"Job": "Mechanic",
}

enum Tile {
	BRICK,
	FLOOR,
	SPIKE,
	TELEPORT,
}
