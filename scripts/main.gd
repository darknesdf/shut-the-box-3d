extends Node3D
## Shut the Box 3D - game controller. Everything is built in code.

const TileScript = preload("res://scripts/tile.gd")
const DieScript = preload("res://scripts/die.gd")

enum State { IDLE, ROLLING, SELECTING, ANIMATING, OVER }

const TILE_SPACING := 1.3
const TILE_Z := -2.6
const SAVE_PATH := "user://shutbox.cfg"
const CAM_TARGET := Vector3(0, 0, -0.9)
const GOLD := Color(1.0, 0.82, 0.35)
const RED := Color(1.0, 0.38, 0.34)

var state: int = State.IDLE
var tiles: Array = []
var dice: Array = []
var camera: Camera3D
var confetti: CPUParticles3D

var roll_total := 0
var selected_sum := 0
var pending_dice := 0
var best_score := -1
var wins := 0
var games := 0
var gen := 0
var time_acc := 0.0
var cam_shake := 0.0
var last_roll_text := ""

var info_label: Label
var sub_label: Label
var roll_btn: Button
var clear_btn: Button
var new_btn: Button
var flash: ColorRect
var overlay: Control
var overlay_panel: PanelContainer
var overlay_title: Label
var overlay_detail: Label
var again_btn: Button
var _info_tween: Tween


func _ready() -> void:
	randomize()
	_build_world()
	_build_tiles()
	_build_dice()
	_build_ui()
	_load_save()
	_new_game()


func _process(delta: float) -> void:
	time_acc += delta
	camera.position = Vector3(sin(time_acc * 0.35) * 0.35, 9.0 + sin(time_acc * 0.27) * 0.15, 8.5)
	if cam_shake > 0.0:
		camera.position += Vector3(randf_range(-1.0, 1.0), randf_range(-1.0, 1.0), 0.0) * cam_shake * 0.25
		cam_shake = maxf(cam_shake - delta * 1.4, 0.0)
	camera.look_at(CAM_TARGET, Vector3.UP)


## Wide phones fit by height, narrower screens fit by width, so the whole tray stays visible.
func _fit_camera() -> void:
	var s := get_viewport().get_visible_rect().size
	if s.x / maxf(s.y, 1.0) >= 1.7:
		camera.keep_aspect = Camera3D.KEEP_HEIGHT
		camera.fov = 46.0
	else:
		camera.keep_aspect = Camera3D.KEEP_WIDTH
		camera.fov = 62.0


# ---------------------------------------------------------------- world

func _mat(c: Color, rough := 0.6) -> StandardMaterial3D:
	var m := StandardMaterial3D.new()
	m.albedo_color = c
	m.roughness = rough
	return m


func _box(size: Vector3, pos: Vector3, c: Color, rough := 0.6) -> MeshInstance3D:
	var mi := MeshInstance3D.new()
	var bm := BoxMesh.new()
	bm.size = size
	mi.mesh = bm
	mi.material_override = _mat(c, rough)
	mi.position = pos
	add_child(mi)
	return mi


func _build_world() -> void:
	var env := Environment.new()
	env.background_mode = Environment.BG_COLOR
	env.background_color = Color(0.07, 0.09, 0.13)
	env.ambient_light_source = Environment.AMBIENT_SOURCE_COLOR
	env.ambient_light_color = Color(0.72, 0.76, 0.92)
	env.ambient_light_energy = 0.35
	var we := WorldEnvironment.new()
	we.environment = env
	add_child(we)

	var sun := DirectionalLight3D.new()
	sun.rotation_degrees = Vector3(-55, -25, 0)
	sun.light_energy = 0.7
	sun.shadow_enabled = true
	sun.directional_shadow_max_distance = 30.0
	add_child(sun)

	camera = Camera3D.new()
	camera.current = true
	add_child(camera)
	camera.position = Vector3(0, 9.0, 8.5)
	camera.look_at(CAM_TARGET, Vector3.UP)
	_fit_camera()
	get_viewport().size_changed.connect(_fit_camera)

	# table + tray
	var floor_mi := MeshInstance3D.new()
	var pm := PlaneMesh.new()
	pm.size = Vector2(60, 60)
	floor_mi.mesh = pm
	floor_mi.material_override = _mat(Color(0.09, 0.1, 0.13), 0.9)
	floor_mi.position = Vector3(0, -0.4, 0)
	add_child(floor_mi)

	var wood := Color(0.42, 0.25, 0.13)
	_box(Vector3(14.4, 0.4, 9.2), Vector3(0, -0.2, -0.1), wood, 0.55)        # base
	_box(Vector3(14.4, 1.0, 0.4), Vector3(0, 0.5, -3.5), wood.darkened(0.1)) # back wall
	_box(Vector3(0.4, 0.8, 9.2), Vector3(-7.0, 0.4, -0.1), wood.darkened(0.1))
	_box(Vector3(0.4, 0.8, 9.2), Vector3(7.0, 0.4, -0.1), wood.darkened(0.1))
	_box(Vector3(14.4, 0.5, 0.4), Vector3(0, 0.25, 4.3), wood.darkened(0.1))
	_box(Vector3(13.2, 0.04, 5.8), Vector3(0, 0.02, 1.4), Color(0.05, 0.2, 0.11), 0.95)  # felt
	_box(Vector3(13.2, 0.06, 1.8), Vector3(0, 0.03, -2.3), Color(0.30, 0.17, 0.09), 0.6) # tile ledge

	# confetti
	confetti = CPUParticles3D.new()
	confetti.amount = 160
	confetti.one_shot = true
	confetti.emitting = false
	confetti.lifetime = 2.6
	confetti.explosiveness = 0.95
	var cq := QuadMesh.new()
	cq.size = Vector2(0.2, 0.2)
	var cm := StandardMaterial3D.new()
	cm.vertex_color_use_as_albedo = true
	cm.cull_mode = BaseMaterial3D.CULL_DISABLED
	cm.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	cq.material = cm
	confetti.mesh = cq
	confetti.direction = Vector3.UP
	confetti.spread = 55.0
	confetti.initial_velocity_min = 5.0
	confetti.initial_velocity_max = 9.5
	confetti.gravity = Vector3(0, -9.0, 0)
	confetti.angular_velocity_min = -360.0
	confetti.angular_velocity_max = 360.0
	var g := Gradient.new()
	g.offsets = PackedFloat32Array([0.0, 0.25, 0.5, 0.75, 1.0])
	g.colors = PackedColorArray([
		Color(1, 0.35, 0.35), Color(1, 0.85, 0.3), Color(0.4, 0.9, 0.5),
		Color(0.4, 0.7, 1), Color(0.85, 0.5, 1)])
	confetti.color_initial_ramp = g
	confetti.position = Vector3(0, 1.0, 0.5)
	add_child(confetti)


func _build_tiles() -> void:
	for i in range(9):
		var t = TileScript.new()
		add_child(t)
		t.setup(i + 1)
		t.position = Vector3((i - 4) * TILE_SPACING, 0.06, TILE_Z)
		tiles.append(t)


func _build_dice() -> void:
	for i in range(2):
		var d = DieScript.new()
		add_child(d)
		d.visible = false
		d.position = Vector3(0, 0.5, 2.0)
		dice.append(d)


# ------------------------------------------------------------------- ui

func _style_button(b: Button, col: Color, size := 38) -> void:
	for st in ["normal", "hover", "pressed", "disabled"]:
		var sb := StyleBoxFlat.new()
		var c := col
		if st == "hover":
			c = col.lightened(0.12)
		elif st == "pressed":
			c = col.darkened(0.15)
		elif st == "disabled":
			c = col.darkened(0.5)
		sb.bg_color = c
		sb.set_corner_radius_all(22)
		sb.content_margin_left = 24
		sb.content_margin_right = 24
		sb.shadow_size = 6
		sb.shadow_color = Color(0, 0, 0, 0.45)
		b.add_theme_stylebox_override(st, sb)
	b.add_theme_font_size_override("font_size", size)
	b.add_theme_color_override("font_color", Color.WHITE)
	b.add_theme_color_override("font_hover_color", Color.WHITE)
	b.add_theme_color_override("font_pressed_color", Color.WHITE)
	b.add_theme_color_override("font_disabled_color", Color(1, 1, 1, 0.45))


func _style_label(l: Label, size: int) -> void:
	l.add_theme_font_size_override("font_size", size)
	l.add_theme_color_override("font_color", Color.WHITE)
	l.add_theme_color_override("font_outline_color", Color(0, 0, 0, 0.85))
	l.add_theme_constant_override("outline_size", 8)
	l.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER


func _build_ui() -> void:
	var layer := CanvasLayer.new()
	add_child(layer)
	var root := Control.new()
	root.mouse_filter = Control.MOUSE_FILTER_IGNORE
	layer.add_child(root)
	root.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)

	info_label = Label.new()
	_style_label(info_label, 34)
	root.add_child(info_label)
	info_label.set_anchors_and_offsets_preset(Control.PRESET_TOP_WIDE)
	info_label.offset_top = 14
	info_label.offset_bottom = 70

	sub_label = Label.new()
	_style_label(sub_label, 24)
	sub_label.add_theme_color_override("font_color", Color(0.85, 0.9, 1.0))
	root.add_child(sub_label)
	sub_label.set_anchors_and_offsets_preset(Control.PRESET_TOP_WIDE)
	sub_label.offset_top = 70
	sub_label.offset_bottom = 110

	var hb := HBoxContainer.new()
	hb.mouse_filter = Control.MOUSE_FILTER_IGNORE
	hb.alignment = BoxContainer.ALIGNMENT_CENTER
	hb.add_theme_constant_override("separation", 20)
	root.add_child(hb)
	hb.set_anchors_and_offsets_preset(Control.PRESET_BOTTOM_WIDE)
	hb.offset_top = -120
	hb.offset_bottom = -24

	roll_btn = Button.new()
	roll_btn.text = "ROLL"
	roll_btn.custom_minimum_size = Vector2(300, 90)
	_style_button(roll_btn, Color(0.85, 0.45, 0.1), 42)
	roll_btn.pressed.connect(_on_roll)
	hb.add_child(roll_btn)

	clear_btn = Button.new()
	clear_btn.text = "Clear"
	clear_btn.custom_minimum_size = Vector2(170, 90)
	_style_button(clear_btn, Color(0.45, 0.35, 0.6), 32)
	clear_btn.pressed.connect(_clear_selection)
	hb.add_child(clear_btn)

	new_btn = Button.new()
	new_btn.text = "New game"
	new_btn.custom_minimum_size = Vector2(220, 90)
	_style_button(new_btn, Color(0.25, 0.45, 0.75), 32)
	new_btn.pressed.connect(_new_game)
	hb.add_child(new_btn)

	# red flash used when you lose
	flash = ColorRect.new()
	flash.color = Color(0.9, 0.1, 0.1, 0.0)
	flash.mouse_filter = Control.MOUSE_FILTER_IGNORE
	layer.add_child(flash)
	flash.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)

	_build_overlay(layer)


func _build_overlay(layer: CanvasLayer) -> void:
	overlay = Control.new()
	overlay.visible = false
	overlay.mouse_filter = Control.MOUSE_FILTER_STOP
	layer.add_child(overlay)
	overlay.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)

	var dim := ColorRect.new()
	dim.color = Color(0, 0, 0, 0.55)
	dim.mouse_filter = Control.MOUSE_FILTER_IGNORE
	overlay.add_child(dim)
	dim.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)

	var center := CenterContainer.new()
	center.mouse_filter = Control.MOUSE_FILTER_IGNORE
	overlay.add_child(center)
	center.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)

	overlay_panel = PanelContainer.new()
	var sb := StyleBoxFlat.new()
	sb.bg_color = Color(0.1, 0.11, 0.16, 0.96)
	sb.set_corner_radius_all(32)
	sb.set_border_width_all(3)
	sb.border_color = Color(1, 1, 1, 0.18)
	sb.content_margin_left = 56
	sb.content_margin_right = 56
	sb.content_margin_top = 36
	sb.content_margin_bottom = 40
	sb.shadow_size = 24
	sb.shadow_color = Color(0, 0, 0, 0.6)
	overlay_panel.add_theme_stylebox_override("panel", sb)
	center.add_child(overlay_panel)

	var vb := VBoxContainer.new()
	vb.add_theme_constant_override("separation", 18)
	overlay_panel.add_child(vb)

	overlay_title = Label.new()
	_style_label(overlay_title, 76)
	vb.add_child(overlay_title)

	overlay_detail = Label.new()
	_style_label(overlay_detail, 30)
	overlay_detail.add_theme_color_override("font_color", Color(0.88, 0.92, 1.0))
	overlay_detail.add_theme_constant_override("outline_size", 0)
	vb.add_child(overlay_detail)

	again_btn = Button.new()
	again_btn.text = "Play again"
	again_btn.custom_minimum_size = Vector2(300, 90)
	_style_button(again_btn, Color(0.85, 0.45, 0.1), 40)
	again_btn.pressed.connect(_new_game)
	again_btn.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	vb.add_child(again_btn)


func _show_overlay(title: String, detail: String, col: Color) -> void:
	overlay_title.text = title
	overlay_title.add_theme_color_override("font_color", col)
	overlay_detail.text = detail
	overlay.modulate.a = 0.0
	overlay_panel.scale = Vector2(0.8, 0.8)
	overlay.visible = true
	await get_tree().process_frame
	overlay_panel.pivot_offset = overlay_panel.size * 0.5
	var tw := create_tween().set_parallel(true)
	tw.tween_property(overlay, "modulate:a", 1.0, 0.35).set_trans(Tween.TRANS_SINE)
	tw.tween_property(overlay_panel, "scale", Vector2.ONE, 0.5) \
		.set_trans(Tween.TRANS_BACK).set_ease(Tween.EASE_OUT)


func _hide_overlay() -> void:
	overlay.visible = false


## Set the headline text with a small pop so changes feel alive.
func _info(text: String) -> void:
	info_label.text = text
	info_label.pivot_offset = info_label.size * 0.5
	if _info_tween and _info_tween.is_valid():
		_info_tween.kill()
	info_label.scale = Vector2(1.07, 1.07)
	_info_tween = create_tween()
	_info_tween.tween_property(info_label, "scale", Vector2.ONE, 0.22) \
		.set_trans(Tween.TRANS_BACK).set_ease(Tween.EASE_OUT)


func _open_sum() -> int:
	var s := 0
	for t in tiles:
		if t.is_open:
			s += t.number
	return s


func _update_sub() -> void:
	var best_txt := "-" if best_score < 0 else str(best_score)
	sub_label.text = "Score now: %d     Best: %s     Wins: %d / %d" % [_open_sum(), best_txt, wins, games]


# ----------------------------------------------------------------- game

func _new_game() -> void:
	gen += 1
	state = State.IDLE
	selected_sum = 0
	roll_total = 0
	pending_dice = 0
	confetti.emitting = false
	cam_shake = 0.0
	flash.color.a = 0.0
	_hide_overlay()
	for i in range(tiles.size()):
		tiles[i].reset(i * 0.05)
	for d in dice:
		d.stop()
		for c in d.landed.get_connections():
			d.landed.disconnect(c["callable"])
		d.visible = false
	roll_btn.disabled = false
	clear_btn.disabled = true
	_info("Tap ROLL to throw the dice")
	_update_sub()


func _on_roll() -> void:
	if state != State.IDLE:
		return
	state = State.ROLLING
	roll_btn.disabled = true
	clear_btn.disabled = true
	var one_die: bool = (not tiles[6].is_open) and (not tiles[7].is_open) and (not tiles[8].is_open)
	var n := 1 if one_die else 2
	pending_dice = n
	roll_total = 0
	var vals: Array = []
	_info("Rolling...")

	var spots := [Vector3(-1.4, 0.5, 1.9), Vector3(1.4, 0.5, 1.7)]
	if n == 1:
		spots = [Vector3(0, 0.5, 1.8)]
	for i in range(dice.size()):
		var d = dice[i]
		if i < n:
			var v := randi_range(1, 6)
			vals.append(v)
			roll_total += v
			d.visible = true
			var to: Vector3 = spots[i] + Vector3(randf_range(-0.4, 0.4), 0.0, randf_range(-0.3, 0.3))
			var side := -1.0 if i == 0 else 1.0
			var from := Vector3(side * 9.0, 4.0, 5.0)
			d.landed.connect(_on_die_landed.bind(gen), CONNECT_ONE_SHOT)
			d.roll_to(v, from, to, 1.7 + 0.25 * i)
		else:
			d.visible = false
	var parts := PackedStringArray()
	for v in vals:
		parts.append(str(v))
	last_roll_text = str(roll_total)
	if n > 1:
		last_roll_text = " + ".join(parts) + " = " + str(roll_total)


func _on_die_landed(token: int) -> void:
	if token != gen:
		return  # a die from a game that has since been reset
	pending_dice -= 1
	if pending_dice == 0:
		_after_roll()


func _after_roll() -> void:
	var my := gen
	await get_tree().create_timer(0.35).timeout
	if my != gen:
		return
	if not _has_move():
		_game_over(false)
	else:
		state = State.SELECTING
		_update_selection_text()


func _update_selection_text() -> void:
	_info("Rolled %s   |   picked %d / %d" % [last_roll_text, selected_sum, roll_total])
	clear_btn.disabled = state != State.SELECTING or selected_sum == 0


func _has_move() -> bool:
	var open: Array = []
	for t in tiles:
		if t.is_open:
			open.append(t.number)
	for mask in range(1, 1 << open.size()):
		var s := 0
		for i in range(open.size()):
			if mask & (1 << i):
				s += open[i]
		if s == roll_total:
			return true
	return false


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		_pick(event.position)


func _pick(pos: Vector2) -> void:
	if state != State.SELECTING:
		return
	var from := camera.project_ray_origin(pos)
	var to := from + camera.project_ray_normal(pos) * 100.0
	var q := PhysicsRayQueryParameters3D.create(from, to)
	var hit := get_world_3d().direct_space_state.intersect_ray(q)
	if hit.is_empty():
		return
	var col: Object = hit.collider
	if col.has_meta("tile"):
		_toggle(col.get_meta("tile"))


func _toggle(t) -> void:
	if not t.is_open:
		return
	if t.selected:
		t.set_selected(false)
		selected_sum -= t.number
	elif selected_sum + t.number > roll_total:
		t.shake()
		Input.vibrate_handheld(40)
		return
	else:
		t.set_selected(true)
		selected_sum += t.number
	_update_selection_text()
	if selected_sum == roll_total:
		_commit()


func _clear_selection() -> void:
	if state != State.SELECTING:
		return
	for t in tiles:
		if t.selected:
			t.set_selected(false)
	selected_sum = 0
	_update_selection_text()


func _commit() -> void:
	state = State.ANIMATING
	clear_btn.disabled = true
	var my := gen
	await get_tree().create_timer(0.3).timeout
	if my != gen:
		return
	var delay := 0.0
	for t in tiles:
		if t.selected:
			t.flip_down(delay)
			delay += 0.16
	selected_sum = 0
	_update_sub()
	await get_tree().create_timer(delay + 1.0).timeout
	if my != gen:
		return
	if _open_sum() == 0:
		_game_over(true)
	else:
		state = State.IDLE
		roll_btn.disabled = false
		_info("Nice! Roll again")


func _game_over(win: bool) -> void:
	state = State.OVER
	roll_btn.disabled = true
	clear_btn.disabled = true
	var my := gen
	var score := _open_sum()
	games += 1
	if win:
		wins += 1
	if best_score < 0 or score < best_score:
		best_score = score
	_save()
	_update_sub()

	if win:
		confetti.restart()
		confetti.emitting = true
		Input.vibrate_handheld(120)
		_info("SHUT THE BOX!")
		await get_tree().create_timer(1.2).timeout
		if my != gen:
			return
		_show_overlay("BOX SHUT!", "Perfect score: 0\nBest: 0   Wins: %d / %d" % [wins, games], GOLD)
	else:
		_info("No combination makes %d" % roll_total)
		Input.vibrate_handheld(300)
		cam_shake = 0.7
		flash.color.a = 0.4
		create_tween().tween_property(flash, "color:a", 0.0, 0.8).set_trans(Tween.TRANS_SINE)
		var d := 0.0
		for t in tiles:
			if t.is_open:
				t.mark_lost(d)
				d += 0.06
		await get_tree().create_timer(1.3).timeout
		if my != gen:
			return
		_show_overlay("YOU LOST",
			"Rolled %s - nothing fits.\nScore: %d (lower is better)\nBest: %d   Wins: %d / %d"
			% [last_roll_text, score, best_score, wins, games], RED)


# ----------------------------------------------------------------- save

func _load_save() -> void:
	var cf := ConfigFile.new()
	if cf.load(SAVE_PATH) == OK:
		best_score = int(cf.get_value("stats", "best", -1))
		wins = int(cf.get_value("stats", "wins", 0))
		games = int(cf.get_value("stats", "games", 0))


func _save() -> void:
	var cf := ConfigFile.new()
	cf.set_value("stats", "best", best_score)
	cf.set_value("stats", "wins", wins)
	cf.set_value("stats", "games", games)
	cf.save(SAVE_PATH)
