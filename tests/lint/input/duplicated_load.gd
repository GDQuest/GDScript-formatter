#region Good
const GOOD_PRELOAD_1 = preload("res://some/path1.tscn")
const GOOD_PRELOAD_2 = preload("res://some/path2.tscn")

var good_load_1 = load("res://another/path1.tscn")
var good_load_2 = load("res://another/path2.tscn")

var good_scene_path = "res://some/path3.tscn"
var good_load_3 = load(good_scene_path)
var good_load_4 = load(good_scene_path)

#endregion


#region Bad
const BAD_PRELOAD_1 = preload("res://duplicate/path.tscn")
const BAD_PRELOAD_2 = preload("res://duplicate/path.tscn")

var bad_load_1 = load("res://another/duplicate/path.tscn")
var bad_load_2 = load("res://another/duplicate/path.tscn")

#endregion