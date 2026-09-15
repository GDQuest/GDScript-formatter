func add_road_segment(v1: Vector3, v2: Vector3, v3: Vector3, v4: Vector3, v5: Vector3, v6: Vector3) -> void:
	var uv_quads_1 := HexMesh.generate_quad_uv(0, 1, 0, 0)
	add_quad(
			HexMesh.Vertex.new(v1, HexMesh.Vertex.NO_COLOR, uv_quads_1.get(0)),
			HexMesh.Vertex.new(v2, HexMesh.Vertex.NO_COLOR, uv_quads_1.get(1)),
			HexMesh.Vertex.new(v4, HexMesh.Vertex.NO_COLOR, uv_quads_1.get(2)),
			HexMesh.Vertex.new(v5, HexMesh.Vertex.NO_COLOR, uv_quads_1.get(3))
	)
