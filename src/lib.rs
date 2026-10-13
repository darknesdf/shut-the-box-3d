//! Shut The Box 3D - Rust + Bevy rewrite of the original Godot project.

mod anim;
mod confetti;
mod die;
mod game;
mod input;
mod logic;
mod save;
mod tile;
mod ui;
mod world;

use bevy::prelude::*;

#[bevy_main]
pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Shut The Box 3D".into(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.07, 0.09, 0.13)))
        .insert_resource(AmbientLight {
            color: Color::srgb(0.72, 0.76, 0.92),
            brightness: 350.0,
        })
        .add_plugins((
            world::WorldPlugin,
            tile::TilePlugin,
            die::DicePlugin,
            game::GamePlugin,
            input::InputPlugin,
            ui::UiPlugin,
            confetti::ConfettiPlugin,
        ))
        .run();
}
