//! 3D die: body cube + 6 textured quads, thrown in an arc with damped bounces.
//! Opposite faces sum to 7: +Y=1, -Y=6, +X=3, -X=4, +Z=2, -Z=5.

use crate::game::Game;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use rand::Rng;
use std::f32::consts::{FRAC_PI_2, PI, TAU};

const FALL_FRAC: f32 = 0.30; // share of the roll spent on the first fall
const BOUNCES: i32 = 4;
const RESTITUTION: f32 = 0.55;
const HOP_HEIGHT: f32 = 1.6;

struct Roll {
    fin: Quat,
    axis: Vec3,
    spins: f32,
    from: Vec3,
    to: Vec3,
    dur: f32,
    t: f32,
}

#[derive(Component)]
pub struct Die {
    pub index: usize,
    pub value: u32,
    roll: Option<Roll>,
}

impl Die {
    fn new(index: usize) -> Self {
        Self { index, value: 1, roll: None }
    }

    /// Cancel a roll in progress (a new game started mid-throw).
    pub fn stop(&mut self) {
        self.roll = None;
    }

    /// Throw the die from `from` to `to` so that face `v` ends up on top.
    pub fn roll_to(&mut self, v: u32, from: Vec3, to: Vec3, dur: f32, tf: &mut Transform) {
        let mut rng = rand::thread_rng();
        let fin = Quat::from_axis_angle(Vec3::Y, rng.gen_range(0.0..TAU)) * base_quat(v);
        let mut axis = Vec3::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0));
        if axis.length() < 0.1 {
            axis = Vec3::X;
        }
        let r = Roll { fin, axis: axis.normalize(), spins: rng.gen_range(5.0..7.5) * PI, from, to, dur, t: 0.0 };
        let (q, p) = pose(&r, 0.0);
        tf.rotation = q;
        tf.translation = p;
        self.value = v;
        self.roll = Some(r);
    }
}

fn base_quat(v: u32) -> Quat {
    match v {
        1 => Quat::IDENTITY,
        6 => Quat::from_axis_angle(Vec3::X, PI),
        3 => Quat::from_axis_angle(Vec3::Z, FRAC_PI_2),
        4 => Quat::from_axis_angle(Vec3::Z, -FRAC_PI_2),
        2 => Quat::from_axis_angle(Vec3::X, -FRAC_PI_2),
        _ => Quat::from_axis_angle(Vec3::X, FRAC_PI_2),
    }
}

/// Height above the landing spot at normalized time t: an accelerating fall, then damped hops.
fn height(r: &Roll, t: f32) -> f32 {
    if t < FALL_FRAC {
        let u0 = t / FALL_FRAC;
        return (r.from.y - r.to.y).max(0.0) * (1.0 - u0 * u0);
    }
    let rem = (t - FALL_FRAC) / (1.0 - FALL_FRAC);
    let total: f32 = (0..BOUNCES).map(|k| RESTITUTION.powi(k)).sum();
    let mut acc = 0.0;
    for k in 0..BOUNCES {
        let w = RESTITUTION.powi(k) / total;
        if rem <= acc + w || k == BOUNCES - 1 {
            let u1 = ((rem - acc) / w).clamp(0.0, 1.0);
            return 4.0 * HOP_HEIGHT * RESTITUTION.powi(2 * k) * u1 * (1.0 - u1);
        }
        acc += w;
    }
    0.0
}

fn pose(r: &Roll, t: f32) -> (Quat, Vec3) {
    let t = t.clamp(0.0, 1.0);
    // spin winds down smoothly and is exactly the target orientation at t = 1
    let spin_e = 1.0 - (1.0 - t).powf(2.6);
    let q = Quat::from_axis_angle(r.axis, r.spins * (1.0 - spin_e)) * r.fin;
    // slide across the table with a soft ease-out while hopping
    let slide_e = 1.0 - (1.0 - t).powf(2.2);
    let mut p = r.from.lerp(r.to, slide_e);
    p.y = r.to.y + height(r, t);
    (q, p)
}

pub struct DicePlugin;

impl Plugin for DicePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_dice).add_systems(Update, die_system);
    }
}

fn die_system(time: Res<Time>, mut game: ResMut<Game>, mut dice: Query<(&mut Die, &mut Transform)>) {
    let dt = time.delta_secs();
    for (mut die, mut tf) in &mut dice {
        let mut landed = false;
        if let Some(r) = die.roll.as_mut() {
            r.t += dt / r.dur;
            let (q, p) = pose(r, r.t);
            tf.rotation = q;
            tf.translation = p;
            landed = r.t >= 1.0;
        }
        if landed {
            die.roll = None;
            game.die_landed();
        }
    }
}

fn setup_dice(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let body_mesh = meshes.add(Cuboid::new(0.96, 0.96, 0.96));
    let body_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.86, 0.83, 0.74),
        perceptual_roughness: 0.4,
        ..default()
    });
    let face_mesh = meshes.add(Rectangle::new(1.0, 1.0));
    let face_mats: Vec<Handle<StandardMaterial>> = (1..=6)
        .map(|v| {
            materials.add(StandardMaterial {
                base_color_texture: Some(images.add(make_face(v))),
                alpha_mode: AlphaMode::Mask(0.5),
                perceptual_roughness: 0.3,
                ..default()
            })
        })
        .collect();

    // (value, position, rotation)
    let faces = [
        (1usize, Vec3::new(0.0, 0.5, 0.0), Quat::from_axis_angle(Vec3::X, -FRAC_PI_2)),
        (6, Vec3::new(0.0, -0.5, 0.0), Quat::from_axis_angle(Vec3::X, FRAC_PI_2)),
        (3, Vec3::new(0.5, 0.0, 0.0), Quat::from_axis_angle(Vec3::Y, FRAC_PI_2)),
        (4, Vec3::new(-0.5, 0.0, 0.0), Quat::from_axis_angle(Vec3::Y, -FRAC_PI_2)),
        (2, Vec3::new(0.0, 0.0, 0.5), Quat::IDENTITY),
        (5, Vec3::new(0.0, 0.0, -0.5), Quat::from_axis_angle(Vec3::Y, PI)),
    ];

    for i in 0..2 {
        let (body_mesh, body_mat, face_mesh) = (body_mesh.clone(), body_mat.clone(), face_mesh.clone());
        let face_mats = face_mats.clone();
        commands
            .spawn((Die::new(i), Transform::from_xyz(0.0, 0.5, 2.0), Visibility::Hidden))
            .with_children(|p| {
                p.spawn((Mesh3d(body_mesh), MeshMaterial3d(body_mat)));
                for (v, pos, rot) in faces {
                    p.spawn((
                        Mesh3d(face_mesh.clone()),
                        MeshMaterial3d(face_mats[v - 1].clone()),
                        Transform::from_translation(pos * 1.002).with_rotation(rot),
                        bevy::pbr::NotShadowCaster,
                    ));
                }
            });
    }
}

/// Procedurally drawn ivory face with rounded corners and anti-aliased pips.
fn make_face(v: u32) -> Image {
    const S: usize = 128;
    let ivory = [0.97f32, 0.95, 0.88];
    let r = 22.0f32;
    let mut px = vec![[0.0f32; 4]; S * S];
    for y in 0..S {
        for x in 0..S {
            let cx = (x as f32).clamp(r, S as f32 - 1.0 - r);
            let cy = (y as f32).clamp(r, S as f32 - 1.0 - r);
            let d = ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt();
            px[y * S + x] = if d > r {
                [0.0, 0.0, 0.0, 0.0]
            } else if d > r - 5.0 {
                [ivory[0] * 0.86, ivory[1] * 0.86, ivory[2] * 0.86, 1.0]
            } else {
                [ivory[0], ivory[1], ivory[2], 1.0]
            };
        }
    }

    let grid: &[(u32, u32)] = match v {
        1 => &[(1, 1)],
        2 => &[(0, 0), (2, 2)],
        3 => &[(0, 0), (1, 1), (2, 2)],
        4 => &[(0, 0), (2, 0), (0, 2), (2, 2)],
        5 => &[(0, 0), (2, 0), (1, 1), (0, 2), (2, 2)],
        _ => &[(0, 0), (0, 1), (0, 2), (2, 0), (2, 1), (2, 2)],
    };
    let pip = if v == 1 { [0.8f32, 0.1, 0.1] } else { [0.08, 0.08, 0.1] };
    let pr = if v == 1 { 14.0f32 } else { 11.0 };
    for &(gx, gy) in grid {
        let (pcx, pcy) = (S as f32 * (0.25 + 0.25 * gx as f32), S as f32 * (0.25 + 0.25 * gy as f32));
        for y in (pcy - pr - 2.0) as usize..(pcy + pr + 3.0) as usize {
            for x in (pcx - pr - 2.0) as usize..(pcx + pr + 3.0) as usize {
                let dist = ((x as f32 - pcx).powi(2) + (y as f32 - pcy).powi(2)).sqrt();
                let cov = (pr + 0.5 - dist).clamp(0.0, 1.0);
                if cov > 0.0 {
                    let b = &mut px[y * S + x];
                    for k in 0..3 {
                        b[k] += (pip[k] - b[k]) * cov;
                    }
                }
            }
        }
    }
    let data: Vec<u8> = px.iter().flat_map(|c| c.map(|f| (f.clamp(0.0, 1.0) * 255.0) as u8)).collect();
    Image::new(
        Extent3d { width: S as u32, height: S as u32, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}
