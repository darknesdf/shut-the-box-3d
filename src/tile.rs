//! Numbered tile: hinge entity -> mesh_root (pose animation) -> inner (box, plates, picking body).

use crate::anim::{Channel, Ease};
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::f32::consts::FRAC_PI_2;

pub const TILE_W: f32 = 1.1;
pub const TILE_H: f32 = 1.5;
pub const TILE_T: f32 = 0.18;
pub const TILE_SPACING: f32 = 1.3;
pub const TILE_Z: f32 = -2.6;
const FLAT: f32 = FRAC_PI_2;

const CREAM: [f32; 3] = [0.96, 0.92, 0.82];
const GOLD: [f32; 3] = [1.0, 0.8, 0.3];
const LOSE: [f32; 3] = [0.88, 0.36, 0.32];

#[derive(Component)]
pub struct Tile {
    pub number: u8,
    plate_mat: Handle<StandardMaterial>,
    mesh_root: Entity,
    z: Channel,
    x: Channel,
    y: Channel,
    scale: Channel,
    rot: Channel,
    gold: Channel,
    lose: Channel,
    last_tint: [f32; 3],
}

#[derive(Component)]
pub struct MeshRoot;

/// Marker on the box centre, used for ray picking.
#[derive(Component)]
pub struct TileBody {
    pub number: u8,
}

impl Tile {
    fn busy(&self) -> bool {
        self.z.busy()
            || self.x.busy()
            || self.y.busy()
            || self.scale.busy()
            || self.rot.busy()
            || self.gold.busy()
            || self.lose.busy()
    }

    pub fn set_selected(&mut self, on: bool) {
        // lifted card leans a little towards the camera
        self.rot.go(if on { 0.10 } else { 0.0 }, 0.28, 0.0, Ease::BackOut);
        self.z.go(if on { 0.45 } else { 0.0 }, 0.28, 0.0, Ease::BackOut);
        self.scale.go(if on { 1.07 } else { 1.0 }, 0.28, 0.0, Ease::BackOut);
        self.gold.go(if on { 1.0 } else { 0.0 }, 0.22, 0.0, Ease::SineInOut);
    }

    /// Damped side-to-side wiggle ("that doesn't fit").
    pub fn shake(&mut self) {
        self.x.clear();
        let mut amp = 0.14;
        for i in 0..5 {
            self.x.push(if i % 2 == 0 { amp } else { -amp }, 0.05, 0.0, Ease::SineInOut);
            amp *= 0.6;
        }
        self.x.push(0.0, 0.05, 0.0, Ease::SineInOut);
    }

    /// Fold onto the table: ease-in fall, then a couple of shrinking rebounds.
    pub fn flip_down(&mut self, delay: f32) {
        self.z.go(0.0, 0.22, delay, Ease::QuadOut);
        self.scale.go(1.0, 0.22, delay, Ease::QuadOut);
        self.x.go(0.0, 0.1, delay, Ease::SineInOut);
        // small hop up before the fall
        self.y.clear();
        self.y.push(0.28, 0.12, delay, Ease::QuadOut);
        self.y.push(0.0, 0.26, 0.0, Ease::QuadIn);
        self.rot.clear();
        self.rot.push(FLAT, 0.34, delay + 0.08, Ease::QuadIn);
        self.rot.push(FLAT - 0.28, 0.13, 0.0, Ease::QuadOut);
        self.rot.push(FLAT, 0.13, 0.0, Ease::QuadIn);
        self.rot.push(FLAT - 0.07, 0.07, 0.0, Ease::QuadOut);
        self.rot.push(FLAT, 0.07, 0.0, Ease::QuadIn);
    }

    /// Tiles still standing when the player loses: tint red and slump forward.
    pub fn mark_lost(&mut self, delay: f32) {
        self.lose.go(1.0, 0.4, delay, Ease::SineInOut);
        self.z.go(0.0, 0.25, 0.0, Ease::SineInOut);
        self.x.go(0.0, 0.25, 0.0, Ease::SineInOut);
        self.y.go(0.0, 0.25, 0.0, Ease::SineInOut);
        self.scale.go(1.0, 0.25, 0.0, Ease::SineInOut);
        self.rot.go(0.24, 0.5, delay, Ease::QuadOut);
    }

    pub fn reset(&mut self, delay: f32) {
        self.z.go(0.0, 0.3, delay, Ease::SineInOut);
        self.x.go(0.0, 0.3, delay, Ease::SineInOut);
        self.y.go(0.0, 0.3, delay, Ease::SineInOut);
        self.scale.go(1.0, 0.3, delay, Ease::SineInOut);
        self.gold.go(0.0, 0.3, delay, Ease::SineInOut);
        self.lose.go(0.0, 0.3, delay, Ease::SineInOut);
        self.rot.go(0.0, 0.6, delay, Ease::BackOut);
    }
}

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_tiles).add_systems(Update, tile_anim_system);
    }
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

fn setup_tiles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let body_mesh = meshes.add(Cuboid::new(TILE_W, TILE_H, TILE_T));
    let plate_mesh = meshes.add(Rectangle::new(TILE_W - 0.14, TILE_H - 0.14));
    let wood = materials.add(StandardMaterial {
        base_color: Color::srgb(0.36, 0.2, 0.1),
        perceptual_roughness: 0.55,
        ..default()
    });
    let back_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.45, 0.08, 0.12),
        perceptual_roughness: 0.5,
        ..default()
    });

    for i in 0..9u8 {
        let number = i + 1;
        let plate_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(CREAM[0], CREAM[1], CREAM[2]),
            base_color_texture: Some(images.add(digit_image(number))),
            perceptual_roughness: 0.45,
            ..default()
        });

        let mesh_root = commands.spawn((MeshRoot, Transform::default(), Visibility::default())).id();
        let tile = commands
            .spawn((
                Transform::from_xyz((i as f32 - 4.0) * TILE_SPACING, 0.06, TILE_Z),
                Visibility::default(),
            ))
            .add_child(mesh_root)
            .id();
        commands.entity(tile).insert(Tile {
            number,
            plate_mat: plate_mat.clone(),
            mesh_root,
            z: Channel::new(0.0),
            x: Channel::new(0.0),
            y: Channel::new(0.0),
            scale: Channel::new(1.0),
            rot: Channel::new(0.0),
            gold: Channel::new(0.0),
            lose: Channel::new(0.0),
            last_tint: CREAM,
        });

        let (body_mesh, plate_mesh, wood, back_mat) =
            (body_mesh.clone(), plate_mesh.clone(), wood.clone(), back_mat.clone());
        commands.entity(mesh_root).with_children(|p| {
            // inner: base of the tile sits on the hinge, slab behind it
            p.spawn((
                Transform::from_xyz(0.0, TILE_H * 0.5, -TILE_T * 0.5),
                Visibility::default(),
                TileBody { number },
            ))
            .with_children(|p| {
                p.spawn((Mesh3d(body_mesh), MeshMaterial3d(wood)));
                // front card with the number
                p.spawn((
                    Mesh3d(plate_mesh.clone()),
                    MeshMaterial3d(plate_mat),
                    Transform::from_xyz(0.0, 0.0, TILE_T * 0.5 + 0.003),
                    bevy::pbr::NotShadowCaster,
                ));
                // back plate, visible once the tile is shut
                p.spawn((
                    Mesh3d(plate_mesh),
                    MeshMaterial3d(back_mat),
                    Transform::from_xyz(0.0, 0.0, -TILE_T * 0.5 - 0.003)
                        .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
                    bevy::pbr::NotShadowCaster,
                ));
            });
        });
    }
}

fn tile_anim_system(
    time: Res<Time>,
    mut tiles: Query<(&mut Tile, &mut Transform)>,
    mut roots: Query<&mut Transform, (With<MeshRoot>, Without<Tile>)>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    let dt = time.delta_secs();
    for (mut tile, mut tf) in &mut tiles {
        // nothing moving: skip all work (and avoid touching change detection)
        if !tile.busy() {
            continue;
        }
        tile.z.update(dt);
        tile.x.update(dt);
        tile.y.update(dt);
        tile.scale.update(dt);
        tile.rot.update(dt);
        tile.gold.update(dt);
        tile.lose.update(dt);

        tf.rotation = Quat::from_rotation_x(tile.rot.v);
        if let Ok(mut r) = roots.get_mut(tile.mesh_root) {
            r.translation = Vec3::new(tile.x.v, tile.y.v, tile.z.v);
            r.scale = Vec3::splat(tile.scale.v);
        }

        let tint = lerp3(lerp3(CREAM, GOLD, tile.gold.v), LOSE, tile.lose.v);
        if tint != tile.last_tint {
            tile.last_tint = tint;
            if let Some(m) = mats.get_mut(&tile.plate_mat) {
                m.base_color = Color::srgb(tint[0], tint[1], tint[2]);
            }
        }
    }
}

// ---------------------------------------------------------------- number texture

/// 7-segment digit (anti-aliased capsules) on a white card; the plate colour tints it.
fn digit_image(n: u8) -> Image {
    const W: usize = 192;
    const H: usize = 272;
    // bits: a top, b top-right, c bottom-right, d bottom, e bottom-left, f top-left, g middle
    const MASKS: [u8; 10] = [63, 6, 91, 79, 102, 109, 125, 7, 127, 111];
    let mask = MASKS[(n % 10) as usize];

    let (cx, cy, hw, hh, r) = (W as f32 * 0.5, H as f32 * 0.5, 40.0, 72.0, 9.0);
    let tl = Vec2::new(cx - hw, cy - hh);
    let tr = Vec2::new(cx + hw, cy - hh);
    let ml = Vec2::new(cx - hw, cy);
    let mr = Vec2::new(cx + hw, cy);
    let bl = Vec2::new(cx - hw, cy + hh);
    let br = Vec2::new(cx + hw, cy + hh);
    let segs = [(tl, tr), (tr, mr), (mr, br), (bl, br), (ml, bl), (tl, ml), (ml, mr)];

    let ink = [0.16f32, 0.08, 0.04];
    let mut data = Vec::with_capacity(W * H * 4);
    for y in 0..H {
        for x in 0..W {
            let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
            let mut cov = 0.0f32;
            for (i, (a, b)) in segs.iter().enumerate() {
                if mask & (1 << i) != 0 {
                    cov = cov.max((r + 0.5 - seg_dist(p, *a, *b)).clamp(0.0, 1.0));
                }
            }
            for c in ink {
                data.push(((1.0 + (c - 1.0) * cov) * 255.0) as u8);
            }
            data.push(255);
        }
    }
    Image::new(
        Extent3d { width: W as u32, height: H as u32, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

fn seg_dist(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let t = ((p - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0);
    (p - (a + ab * t)).length()
}
