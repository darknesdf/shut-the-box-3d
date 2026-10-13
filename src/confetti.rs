//! Win confetti: a burst of small unlit quads with ballistic motion.

use crate::game::Game;
use bevy::prelude::*;
use rand::Rng;

const AMOUNT: usize = 160;
const LIFETIME: f32 = 2.6;
const GRAVITY: f32 = -9.0;

#[derive(Component)]
struct Confetto {
    vel: Vec3,
    spin: Vec3, // rad/s
    age: f32,
}

#[derive(Resource)]
struct ConfettiAssets {
    mesh: Handle<Mesh>,
    mats: Vec<Handle<StandardMaterial>>,
}

pub struct ConfettiPlugin;

impl Plugin for ConfettiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(Update, confetti_system);
    }
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let colors = [
        Color::srgb(1.0, 0.35, 0.35),
        Color::srgb(1.0, 0.85, 0.3),
        Color::srgb(0.4, 0.9, 0.5),
        Color::srgb(0.4, 0.7, 1.0),
        Color::srgb(0.85, 0.5, 1.0),
    ];
    let mats = colors
        .into_iter()
        .map(|c| {
            materials.add(StandardMaterial {
                base_color: c,
                unlit: true,
                cull_mode: None,
                double_sided: true,
                ..default()
            })
        })
        .collect();
    commands.insert_resource(ConfettiAssets { mesh: meshes.add(Rectangle::new(0.2, 0.2)), mats });
}

fn confetti_system(
    mut commands: Commands,
    time: Res<Time>,
    mut game: ResMut<Game>,
    assets: Res<ConfettiAssets>,
    mut q: Query<(Entity, &mut Transform, &mut Confetto)>,
) {
    let dt = time.delta_secs();
    let mut rng = rand::thread_rng();

    if std::mem::take(&mut game.confetti_clear) {
        for (e, _, _) in &q {
            commands.entity(e).despawn();
        }
    }
    if std::mem::take(&mut game.confetti_burst) {
        let cos_spread = 55f32.to_radians().cos();
        for i in 0..AMOUNT {
            // random direction inside a 55 degree cone around +Y
            let cos_t = rng.gen_range(cos_spread..1.0f32);
            let sin_t = (1.0 - cos_t * cos_t).sqrt();
            let phi = rng.gen_range(0.0..std::f32::consts::TAU);
            let dir = Vec3::new(sin_t * phi.cos(), cos_t, sin_t * phi.sin());
            let speed = rng.gen_range(5.0..9.5);
            let spin = Vec3::new(
                rng.gen_range(-6.3..6.3),
                rng.gen_range(-6.3..6.3),
                rng.gen_range(-6.3..6.3),
            );
            commands.spawn((
                Mesh3d(assets.mesh.clone()),
                MeshMaterial3d(assets.mats[i % assets.mats.len()].clone()),
                Transform::from_xyz(0.0, 1.0, 0.5),
                Confetto { vel: dir * speed, spin, age: 0.0 },
            ));
        }
    }

    for (e, mut tf, mut c) in &mut q {
        c.age += dt;
        if c.age > LIFETIME {
            commands.entity(e).despawn();
            continue;
        }
        c.vel.y += GRAVITY * dt;
        tf.translation += c.vel * dt;
        let s = c.spin * dt;
        tf.rotate(Quat::from_euler(EulerRot::XYZ, s.x, s.y, s.z));
    }
}
