//! Tap / click picking: ray from the camera tested against each tile's oriented box.

use crate::game::{game_system, Actions, Game, Phase};
use crate::tile::{TileBody, TILE_H, TILE_T, TILE_W};
use crate::world::MainCamera;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, pick_system.before(game_system));
    }
}

fn pick_system(
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cam: Query<(&GlobalTransform, &Projection), With<MainCamera>>,
    bodies: Query<(&GlobalTransform, &TileBody)>,
    game: Res<Game>,
    mut act: ResMut<Actions>,
) {
    if game.phase != Phase::Selecting {
        return;
    }
    let Ok(window) = windows.get_single() else { return };
    let Ok((cam_tf, proj)) = cam.get_single() else { return };
    let Projection::Perspective(p) = proj else { return };

    let mut pos = None;
    if mouse.just_pressed(MouseButton::Left) {
        pos = window.cursor_position();
    }
    for t in touches.iter_just_pressed() {
        pos = Some(t.position());
    }
    let Some(pos) = pos else { return };

    // screen -> world ray
    let size = Vec2::new(window.width(), window.height());
    let ndc = Vec2::new(pos.x / size.x * 2.0 - 1.0, 1.0 - pos.y / size.y * 2.0);
    let th = (p.fov * 0.5).tan();
    let dir_local = Vec3::new(ndc.x * th * (size.x / size.y), ndc.y * th, -1.0);
    let (_, rot, origin) = cam_tf.to_scale_rotation_translation();
    let dir = (rot * dir_local).normalize();

    let half = Vec3::new(TILE_W * 0.5, TILE_H * 0.5, (TILE_T + 0.2) * 0.5);
    let mut best: Option<(f32, u8)> = None;
    for (gt, body) in &bodies {
        let inv = gt.compute_matrix().inverse();
        let o = inv.transform_point3(origin);
        let d = inv.transform_vector3(dir);
        if let Some(t) = ray_box(o, d, half) {
            if best.map_or(true, |(bt, _)| t < bt) {
                best = Some((t, body.number));
            }
        }
    }
    if let Some((_, n)) = best {
        act.pick = Some(n);
    }
}

/// Ray vs axis-aligned box centred at the origin (slab method).
fn ray_box(o: Vec3, d: Vec3, half: Vec3) -> Option<f32> {
    let (mut tmin, mut tmax) = (0.0f32, f32::MAX);
    for i in 0..3 {
        if d[i].abs() < 1e-8 {
            if o[i].abs() > half[i] {
                return None;
            }
        } else {
            let t1 = (-half[i] - o[i]) / d[i];
            let t2 = (half[i] - o[i]) / d[i];
            let (a, b) = if t1 < t2 { (t1, t2) } else { (t2, t1) };
            tmin = tmin.max(a);
            tmax = tmax.min(b);
            if tmin > tmax {
                return None;
            }
        }
    }
    Some(tmin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ray_hits_and_misses() {
        let half = Vec3::splat(1.0);
        assert!(ray_box(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0), half).is_some());
        assert!(ray_box(Vec3::new(3.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0), half).is_none());
    }
}
