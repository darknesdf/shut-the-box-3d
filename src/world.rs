//! Camera, light, wooden tray and table.

use crate::game::Game;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::Rng;

pub const CAM_TARGET: Vec3 = Vec3::new(0.0, 0.0, -0.9);

#[derive(Component)]
pub struct MainCamera;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_world).add_systems(Update, camera_system);
    }
}

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection { fov: 46f32.to_radians(), ..default() }),
        Msaa::Sample4,
        Tonemapping::AcesFitted,
        Transform::from_xyz(0.0, 9.0, 8.5).looking_at(CAM_TARGET, Vec3::Y),
        MainCamera,
    ));

    commands.spawn((
        DirectionalLight { illuminance: 7000.0, shadows_enabled: true, ..default() },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::YXZ,
            (-25f32).to_radians(),
            (-55f32).to_radians(),
            0.0,
        )),
        CascadeShadowConfigBuilder { maximum_distance: 30.0, ..default() }.build(),
    ));

    let mut boxm = |size: Vec3, pos: Vec3, c: Color, rough: f32| {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: c,
                perceptual_roughness: rough,
                ..default()
            })),
            Transform::from_translation(pos),
        ));
    };

    let wood = Color::srgb(0.42, 0.25, 0.13);
    let dark = Color::srgb(0.42 * 0.9, 0.25 * 0.9, 0.13 * 0.9);
    boxm(Vec3::new(14.4, 0.4, 9.2), Vec3::new(0.0, -0.2, -0.1), wood, 0.55); // base
    boxm(Vec3::new(14.4, 1.0, 0.4), Vec3::new(0.0, 0.5, -3.5), dark, 0.6); // back wall
    boxm(Vec3::new(0.4, 0.8, 9.2), Vec3::new(-7.0, 0.4, -0.1), dark, 0.6);
    boxm(Vec3::new(0.4, 0.8, 9.2), Vec3::new(7.0, 0.4, -0.1), dark, 0.6);
    boxm(Vec3::new(14.4, 0.5, 0.4), Vec3::new(0.0, 0.25, 4.3), dark, 0.6);
    boxm(Vec3::new(13.2, 0.04, 5.8), Vec3::new(0.0, 0.02, 1.4), Color::srgb(0.05, 0.2, 0.11), 0.95); // felt
    boxm(Vec3::new(13.2, 0.06, 1.8), Vec3::new(0.0, 0.03, -2.3), Color::srgb(0.30, 0.17, 0.09), 0.6); // tile ledge

    // dark table surface
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(60.0, 60.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.09, 0.1, 0.13),
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.4, 0.0),
    ));
}

/// Gentle sway + shake, and the "fit the whole tray on any aspect ratio" logic.
fn camera_system(
    time: Res<Time>,
    mut game: ResMut<Game>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut cam: Query<(&mut Transform, &mut Projection), With<MainCamera>>,
) {
    let Ok((mut tf, mut proj)) = cam.get_single_mut() else { return };
    let t = time.elapsed_secs();
    let mut pos = Vec3::new((t * 0.35).sin() * 0.35, 9.0 + (t * 0.27).sin() * 0.15, 8.5);
    if game.cam_shake > 0.0 {
        let mut rng = rand::thread_rng();
        pos += Vec3::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0), 0.0) * game.cam_shake * 0.25;
        game.cam_shake = (game.cam_shake - time.delta_secs() * 1.4).max(0.0);
    }
    *tf = Transform::from_translation(pos).looking_at(CAM_TARGET, Vec3::Y);

    // Wide screens fit by height (fov_y 46deg); narrower ones fit by width (fov_x 62deg).
    if let (Ok(w), Projection::Perspective(p)) = (windows.get_single(), &mut *proj) {
        let aspect = w.width() / w.height().max(1.0);
        p.fov = if aspect >= 1.7 {
            46f32.to_radians()
        } else {
            2.0 * ((62f32.to_radians() * 0.5).tan() / aspect).atan()
        };
    }
}
