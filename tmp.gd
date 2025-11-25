extends Node

# Accented characters for testing:
# á é í ó ú
# à ê ô ã õ
# ç Ç

var text: String = "pão, direção, ação, mão, coração."


func _ready() -> void:
	print(text)
