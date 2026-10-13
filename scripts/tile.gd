extends Node3D
## One numbered card/tile. This node is the hinge at the tile's base.
## Closing a tile folds it forward onto the table (dark back face up).

const W := 1.1
const H := 1.5
const T := 0.18
const FLAT := PI / 2.0

const CREAM := Color(0.96, 0.92, 0.82)
const GOLD := Color(1.0, 0.8, 0.3)
const LOSE := Color(0.88, 0.36, 0.32)

var number := 1
var is_open := true
var selected := false

var mesh_root: Node3D
var plate_mat: StandardMaterial3D
var _tween: Tween      # pose of mesh_root + plate colour
var _rot_tween: Tween  # hinge rotation


func setup(n: int) -> void:
	number = n
	mesh_root = Node3D.new()
	add_child(mesh_root)

	# inner node: base of the tile sits on the hinge, back of the slab behind it
	var inner := Node3D.new()
	inner.position = Vector3(0, H * 0.5, -T * 0.5)
	mesh_root.add_child(inner)

	var body := MeshInstance3D.new()
	var bm := BoxMesh.new()
	bm.size = Vector3(W, H, T)
	body.mesh = bm
	var wood := StandardMaterial3D.new()
	wood.albedo_color = Color(0.36, 0.2, 0.1)
	wood.roughness = 0.55
	body.material_override = wood
	inner.add_child(body)

	# front plate (cream card)
	var plate := MeshInstance3D.new()
	var qm := QuadMesh.new()
	qm.size = Vector2(W - 0.14, H - 0.14)
	plate.mesh = qm
	plate.position = Vector3(0, 0, T * 0.5 + 0.003)
	plate_mat = StandardMaterial3D.new()
	plate_mat.albedo_color = CREAM
	plate_mat.roughness = 0.45
	plate.material_override = plate_mat
	plate.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	inner.add_child(plate)

	# number
	var lbl := Label3D.new()
	lbl.text = str(n)
	lbl.font_size = 150
	lbl.pixel_size = 0.0055
	lbl.modulate = Color(0.16, 0.08, 0.04)
	lbl.outline_size = 0
	lbl.position = Vector3(0, 0, T * 0.5 + 0.009)
	inner.add_child(lbl)

	# back plate (shown when the tile is closed)
	var back := MeshInstance3D.new()
	var bq := QuadMesh.new()
	bq.size = Vector2(W - 0.14, H - 0.14)
	back.mesh = bq
	back.position = Vector3(0, 0, -T * 0.5 - 0.003)
	back.rotation.y = PI
	var bmat := StandardMaterial3D.new()
	bmat.albedo_color = Color(0.45, 0.08, 0.12)
	bmat.roughness = 0.5
	back.material_override = bmat
	back.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	inner.add_child(back)

	# picking collider
	var sb := StaticBody3D.new()
	var cs := CollisionShape3D.new()
	var bs := BoxShape3D.new()
	bs.size = Vector3(W, H, T + 0.2)
	cs.shape = bs
	sb.add_child(cs)
	sb.set_meta("tile", self)
	inner.add_child(sb)


func _kill() -> void:
	if _tween and _tween.is_valid():
		_tween.kill()
	if _rot_tween and _rot_tween.is_valid():
		_rot_tween.kill()


func set_selected(on: bool) -> void:
	selected = on
	if _tween and _tween.is_valid():
		_tween.kill()
	_tween = create_tween().set_parallel(true)
	_tween.tween_property(mesh_root, "position:z", 0.45 if on else 0.0, 0.28) \
		.set_trans(Tween.TRANS_BACK).set_ease(Tween.EASE_OUT)
	_tween.tween_property(mesh_root, "scale", Vector3.ONE * (1.07 if on else 1.0), 0.28) \
		.set_trans(Tween.TRANS_BACK).set_ease(Tween.EASE_OUT)
	_tween.tween_property(plate_mat, "albedo_color", GOLD if on else CREAM, 0.22) \
		.set_trans(Tween.TRANS_SINE)


## Quick, damped side-to-side wiggle ("that doesn't fit").
func shake() -> void:
	if _tween and _tween.is_valid():
		_tween.kill()
	_tween = create_tween()
	var amp := 0.14
	for i in range(5):
		_tween.tween_property(mesh_root, "position:x", amp * (1.0 if i % 2 == 0 else -1.0), 0.05) \
			.set_trans(Tween.TRANS_SINE)
		amp *= 0.6
	_tween.tween_property(mesh_root, "position:x", 0.0, 0.05).set_trans(Tween.TRANS_SINE)


## Fold down onto the table: ease-in fall, then a couple of shrinking rebounds.
func flip_down(delay: float) -> void:
	is_open = false
	selected = false
	_kill()
	_tween = create_tween().set_parallel(true)
	_tween.tween_property(mesh_root, "position:z", 0.0, 0.22).set_delay(delay) \
		.set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_OUT)
	_tween.tween_property(mesh_root, "scale", Vector3.ONE, 0.22).set_delay(delay) \
		.set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_OUT)
	_tween.tween_property(mesh_root, "position:x", 0.0, 0.1).set_delay(delay)

	_rot_tween = create_tween()
	_rot_tween.tween_interval(delay + 0.08)
	_rot_tween.tween_property(self, "rotation:x", FLAT, 0.34) \
		.set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_IN)
	_rot_tween.tween_property(self, "rotation:x", FLAT - 0.28, 0.13) \
		.set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_OUT)
	_rot_tween.tween_property(self, "rotation:x", FLAT, 0.13) \
		.set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_IN)
	_rot_tween.tween_property(self, "rotation:x", FLAT - 0.07, 0.07) \
		.set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_OUT)
	_rot_tween.tween_property(self, "rotation:x", FLAT, 0.07) \
		.set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_IN)


## Called on the tiles still standing when the player loses: tint red and slump forward.
func mark_lost(delay: float) -> void:
	selected = false
	_kill()
	_tween = create_tween().set_parallel(true)
	_tween.tween_property(plate_mat, "albedo_color", LOSE, 0.4).set_delay(delay) \
		.set_trans(Tween.TRANS_SINE)
	_tween.tween_property(mesh_root, "position", Vector3.ZERO, 0.25)
	_tween.tween_property(mesh_root, "scale", Vector3.ONE, 0.25)
	_rot_tween = create_tween()
	_rot_tween.tween_interval(delay)
	_rot_tween.tween_property(self, "rotation:x", 0.24, 0.5) \
		.set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_OUT)


func reset(delay: float = 0.0) -> void:
	is_open = true
	selected = false
	_kill()
	_tween = create_tween().set_parallel(true)
	_tween.tween_property(mesh_root, "position", Vector3.ZERO, 0.3).set_delay(delay) \
		.set_trans(Tween.TRANS_SINE)
	_tween.tween_property(mesh_root, "scale", Vector3.ONE, 0.3).set_delay(delay)
	_tween.tween_property(plate_mat, "albedo_color", CREAM, 0.3).set_delay(delay)
	_rot_tween = create_tween()
	_rot_tween.tween_interval(delay)
	_rot_tween.tween_property(self, "rotation:x", 0.0, 0.6) \
		.set_trans(Tween.TRANS_BACK).set_ease(Tween.EASE_OUT)
