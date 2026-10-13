//! HUD, buttons, end-of-game overlay and the red "lost" flash (bevy_ui).

use crate::game::{Actions, Game, Phase};
use bevy::prelude::*;

#[derive(Component, Clone, Copy, PartialEq)]
enum Btn {
    Roll,
    Clear,
    New,
    Again,
}

#[derive(Component, Clone, Copy, PartialEq)]
enum UiText {
    Info,
    Sub,
    Title,
    Detail,
}

#[derive(Component)]
struct OverlayRoot;

#[derive(Component)]
struct FlashNode;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui).add_systems(Update, (button_system, ui_update));
    }
}

fn base_color(b: Btn) -> Color {
    match b {
        Btn::Roll | Btn::Again => Color::srgb(0.85, 0.45, 0.1),
        Btn::Clear => Color::srgb(0.45, 0.35, 0.6),
        Btn::New => Color::srgb(0.25, 0.45, 0.75),
    }
}

/// f > 0 lightens towards white, f < 0 darkens towards black.
fn shade(c: Color, f: f32) -> Color {
    let s = c.to_srgba();
    let (target, k) = if f >= 0.0 { (1.0, f) } else { (0.0, -f) };
    Color::srgba(
        s.red + (target - s.red) * k,
        s.green + (target - s.green) * k,
        s.blue + (target - s.blue) * k,
        s.alpha,
    )
}

fn spawn_button(p: &mut ChildBuilder, kind: Btn, label: &str, width: f32, font: f32) {
    p.spawn((
        Button,
        Node {
            width: Val::Px(width),
            height: Val::Px(64.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(base_color(kind)),
        BorderRadius::all(Val::Px(22.0)),
        kind,
    ))
    .with_children(|b| {
        b.spawn((
            Text::new(label),
            TextFont { font_size: font, ..default() },
            TextColor(Color::WHITE),
        ));
    });
}

fn setup_ui(mut commands: Commands) {
    // headline + stats
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(14.0),
            left: Val::Px(0.0),
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|p| {
            p.spawn((
                Text::new(""),
                TextFont { font_size: 26.0, ..default() },
                TextColor(Color::WHITE),
                UiText::Info,
            ));
            p.spawn((
                Text::new(""),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::srgb(0.85, 0.9, 1.0)),
                UiText::Sub,
            ));
        });

    // bottom buttons
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(36.0),
            left: Val::Px(0.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(32.0),
            ..default()
        })
        .with_children(|p| {
            spawn_button(p, Btn::Roll, "ROLL", 210.0, 30.0);
            spawn_button(p, Btn::Clear, "Clear", 130.0, 24.0);
            spawn_button(p, Btn::New, "New game", 170.0, 24.0);
        });

    // red flash when you lose
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.9, 0.1, 0.1, 0.0)),
        FlashNode,
    ));

    // end-of-game overlay
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            Visibility::Hidden,
            ZIndex(10),
            OverlayRoot,
        ))
        .with_children(|p| {
            p.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(18.0),
                    padding: UiRect::axes(Val::Px(56.0), Val::Px(36.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.11, 0.16, 0.96)),
                BorderRadius::all(Val::Px(32.0)),
            ))
            .with_children(|p| {
                p.spawn((
                    Text::new(""),
                    TextFont { font_size: 52.0, ..default() },
                    TextColor(Color::WHITE),
                    UiText::Title,
                ));
                p.spawn((
                    Text::new(""),
                    TextFont { font_size: 22.0, ..default() },
                    TextColor(Color::srgb(0.88, 0.92, 1.0)),
                    TextLayout::new_with_justify(JustifyText::Center),
                    UiText::Detail,
                ));
                spawn_button(p, Btn::Again, "Play again", 220.0, 28.0);
            });
        });
}

fn button_system(
    game: Res<Game>,
    mut act: ResMut<Actions>,
    mut q: Query<(Ref<Interaction>, &Btn, &mut BackgroundColor)>,
) {
    for (inter, kind, mut bg) in &mut q {
        let enabled = match kind {
            Btn::Roll => game.phase == Phase::Idle,
            Btn::Clear => game.phase == Phase::Selecting && game.board.selected_sum > 0,
            Btn::New => true,
            Btn::Again => game.phase == Phase::Over,
        };
        let base = base_color(*kind);
        let color = if !enabled {
            shade(base, -0.5)
        } else {
            match *inter {
                Interaction::Pressed => shade(base, -0.15),
                Interaction::Hovered => shade(base, 0.12),
                Interaction::None => base,
            }
        };
        if bg.0 != color {
            bg.0 = color;
        }
        if enabled && inter.is_changed() && *inter == Interaction::Pressed {
            match kind {
                Btn::Roll => act.roll = true,
                Btn::Clear => act.clear = true,
                Btn::New | Btn::Again => act.new_game = true,
            }
        }
    }
}

fn ui_update(
    game: Res<Game>,
    mut texts: Query<(&mut Text, &mut TextColor, &UiText)>,
    mut overlay: Query<&mut Visibility, With<OverlayRoot>>,
    mut flash: Query<&mut BackgroundColor, With<FlashNode>>,
) {
    let s = game.stats;
    let best = if s.best < 0 { "-".to_string() } else { s.best.to_string() };
    let sub = format!(
        "Score now: {}     Best: {}     Wins: {} / {}",
        game.board.open_sum(),
        best,
        s.wins,
        s.games
    );

    for (mut text, mut color, kind) in &mut texts {
        let new = match kind {
            UiText::Info => Some(game.info.clone()),
            UiText::Sub => Some(sub.clone()),
            UiText::Title => game.overlay.as_ref().map(|o| o.title.clone()),
            UiText::Detail => game.overlay.as_ref().map(|o| o.detail.clone()),
        };
        if let Some(new) = new {
            if text.0 != new {
                text.0 = new;
            }
        }
        if *kind == UiText::Title {
            if let Some(o) = &game.overlay {
                if color.0 != o.color {
                    color.0 = o.color;
                }
            }
        }
    }

    let want = if game.phase == Phase::Over { Visibility::Visible } else { Visibility::Hidden };
    for mut v in &mut overlay {
        if *v != want {
            *v = want;
        }
    }

    let flash_color = Color::srgba(0.9, 0.1, 0.1, game.flash.v);
    for mut bg in &mut flash {
        if bg.0 != flash_color {
            bg.0 = flash_color;
        }
    }
}
