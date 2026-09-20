# Issue 347: https://github.com/GDQuest/GDScript-formatter/issues/347
extends Resource

@export
var test_first: int

@export var test_second: int

## Minimal tiles count in generated chunk
@export_range(1, 30, 1, "prefer_slider")
var min_tiles: int = 3

## Maximal tiles count in generated chunk
@export_range(1, 30, 1, "prefer_slider")
var max_tiles: int = 8

## Jitter probability when generating a chunk
@export_range(0, 0.5, 0.05) var jitter: float = 0.25

## Medium coverage wanted
@export_range(0, 100, 1, "prefer_slider", "suffix: %") var coverage: int = 30
