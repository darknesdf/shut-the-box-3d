extends Node3D
## A 3D die built from a body cube + 6 textured quads (pips drawn procedurally).
## Opposite faces sum to 7: +Y=1, -Y=6, +X=3, -X=4, +Z=2, -Z=5.

signal landed

static var _textures: Array = []

# Roll motion tuning: a fall from the throw height, then BOUNCES damped hops.
const FALL_FRAC := 0.30   # share of the roll spent on the first fall
const BOUNCES := 4
const RESTITUTION := 0.55 # each hop is shorter in time by this factor (height by its square)
const HOP_HEIGHT := 1.6

var value := 1
var _final := Quaternion.IDENTITY
var _axis := Vector3.UP
var _spins := 0.0
var _from := Vector3.ZERO
var _to := Vector3.ZERO
var _tw: Tween


func _ready() -> void:
	_build()


func _build() -> void:
	if _textures.is_empty():
		for v in range(1, 7):
			_textures.append(_make_face(v))

	var body := MeshInstance3D.new()
	var bm := BoxMesh.new()
	bm.size = Vector3(0.96, 0.96, 0.96)
	body.mesh = bm
	var bmat := StandardMaterial3D.new()
	bmat.albedo_color = Color(0.86, 0.83, 0.74)
	bmat.roughness = 0.4
	body.material_override = bmat
	add_child(body)

	# [value, position, rotation (quaternion)]
	var faces := [
		[1, Vector3(0, 0.5, 0), Quaternion(Vector3.RIGHT, -PI / 2)],
		[6, Vector3(0, -0.5, 0), Quaternion(Vector3.RIGHT, PI / 2)],
		[3, Vector3(0.5, 0, 0), Quaternion(Vector3.UP, PI / 2)],
		[4, Vector3(-0.5, 0, 0), Quaternion(Vector3.UP, -PI / 2)],
		[2, Vector3(0, 0, 0.5), Quaternion.IDENTITY],
		[5, Vector3(0, 0, -0.5), Quaternion(Vector3.UP, PI)],
	]
	for f in faces:
		var q := MeshInstance3D.new()
		var qm := QuadMesh.new()
		qm.size = Vector2(1.0, 1.0)
		q.mesh = qm
		var m := StandardMaterial3D.new()
		m.albedo_texture = _textures[f[0] - 1]
		m.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA_SCISSOR
		m.alpha_scissor_threshold = 0.5
		m.roughness = 0.3
		m.texture_filter = BaseMaterial3D.TEXTURE_FILTER_LINEAR_WITH_MIPMAPS_ANISOTROPIC
		q.material_override = m
		q.position = f[1] * 1.002
		q.quaternion = f[2]
		q.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		add_child(q)


static func _make_face(v: int) -> ImageTexture:
	var s := 128
	var img := Image.create(s, s, false, Image.FORMAT_RGBA8)
	var ivory := Color(0.97, 0.95, 0.88)
	var r := 22.0
	for y in range(s):
		for x in range(s):
			var cx := clampf(float(x), r, s - 1 - r)
			var cy := clampf(float(y), r, s - 1 - r)
			var d := Vector2(x - cx, y - cy).length()
			if d > r:
				img.set_pixel(x, y, Color(0, 0, 0, 0))
			elif d > r - 5.0:
				img.set_pixel(x, y, ivory.darkened(0.14))
			else:
				img.set_pixel(x, y, ivory)

	var grid := {
		1: [[1, 1]],
		2: [[0, 0], [2, 2]],
		3: [[0, 0], [1, 1], [2, 2]],
		4: [[0, 0], [2, 0], [0, 2], [2, 2]],
		5: [[0, 0], [2, 0], [1, 1], [0, 2], [2, 2]],
		6: [[0, 0], [0, 1], [0, 2], [2, 0], [2, 1], [2, 2]],
	}
	var pip_col := Color(0.8, 0.1, 0.1) if v == 1 else Color(0.08, 0.08, 0.1)
	var pr := 14.0 if v == 1 else 11.0
	for p in grid[v]:
		var pc := Vector2(s * (0.25 + 0.25 * p[0]), s * (0.25 + 0.25 * p[1]))
		for y in range(int(pc.y - pr - 2), int(pc.y + pr + 3)):
			for x in range(int(pc.x - pr - 2), int(pc.x + pr + 3)):
				var dist := Vector2(x, y).distance_to(pc)
				var cov := clampf(pr + 0.5 - dist, 0.0, 1.0)
				if cov > 0.0:
					var base := img.get_pixel(x, y)
					img.set_pixel(x, y, base.lerp(pip_col, cov))
	img.generate_mipmaps()
	return ImageTexture.create_from_image(img)


static func _base_quat(v: int) -> Quaternion:
	match v:
		1: return Quaternion.IDENTITY
		6: return Quaternion(Vector3.RIGHT, PI)
		3: return Quaternion(Vector3.BACK, PI / 2)
		4: return Quaternion(Vector3.BACK, -PI / 2)
		2: return Quaternion(Vector3.RIGHT, -PI / 2)
		_: return Quaternion(Vector3.RIGHT, PI / 2)


## Cancel a roll in progress (used when a new game starts mid-throw).
func stop() -> void:
	if _tw and _tw.is_valid():
		_tw.kill()


## Throw the die from `from` to `to` so that face `v` ends up on top.
func roll_to(v: int, from: Vector3, to: Vector3, duration: float = 1.7) -> void:
	stop()
	value = v
	_final = Quaternion(Vector3.UP, randf() * TAU) * _base_quat(v)
	_axis = Vector3(randf_range(-1, 1), randf_range(-1, 1), randf_range(-1, 1))
	if _axis.length() < 0.1:
		_axis = Vector3.RIGHT
	_axis = _axis.normalized()
	_spins = randf_range(5.0, 7.5) * PI
	_from = from
	_to = to
	position = from
	_step(0.0)
	_tw = create_tween()
	_tw.tween_method(_step, 0.0, 1.0, duration)
	_tw.finished.connect(func(): landed.emit())


## Height above the landing spot at normalized time t:
## an accelerating fall, then damped parabolic hops (each shorter and quicker).
func _height(t: float) -> float:
	if t < FALL_FRAC:
		var u0 := t / FALL_FRAC
		return maxf(_from.y - _to.y, 0.0) * (1.0 - u0 * u0)
	var rem := (t - FALL_FRAC) / (1.0 - FALL_FRAC)
	var total := 0.0
	for k in range(BOUNCES):
		total += pow(RESTITUTION, float(k))
	var acc := 0.0
	for k in range(BOUNCES):
		var w := pow(RESTITUTION, float(k)) / total
		if rem <= acc + w or k == BOUNCES - 1:
			var u1 := clampf((rem - acc) / w, 0.0, 1.0)
			return 4.0 * HOP_HEIGHT * pow(RESTITUTION, float(2 * k)) * u1 * (1.0 - u1)
		acc += w
	return 0.0


func _step(t: float) -> void:
	# spin winds down smoothly and is exactly the target orientation at t = 1
	var spin_e := 1.0 - pow(1.0 - t, 2.6)
	quaternion = Quaternion(_axis, _spins * (1.0 - spin_e)) * _final
	# slide across the table with a soft ease-out while hopping
	var slide_e := 1.0 - pow(1.0 - t, 2.2)
	var p := _from.lerp(_to, slide_e)
	p.y = _to.y + _height(t)
	position = p
