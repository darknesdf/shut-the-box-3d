//! Game flow: a small state machine driven by timers, mirroring the original GDScript logic.

use crate::anim::{Channel, Ease};
use crate::die::Die;
use crate::logic::{Board, Toggle};
use crate::save::Stats;
use crate::tile::Tile;
use bevy::prelude::*;
use rand::Rng;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Phase {
    Idle,
    Rolling,
    AfterRoll(f32),
    Selecting,
    Committing(f32),
    Animating(f32),
    Ending { win: bool, t: f32 },
    Over,
}

pub struct OverlayInfo {
    pub title: String,
    pub detail: String,
    pub color: Color,
}

#[derive(Resource)]
pub struct Game {
    pub board: Board,
    pub phase: Phase,
    pub stats: Stats,
    pub pending_dice: u32,
    pub last_roll_text: String,
    pub cam_shake: f32,
    pub info: String,
    pub overlay: Option<OverlayInfo>,
    pub flash: Channel,
    pub confetti_burst: bool,
    pub confetti_clear: bool,
}

impl Game {
    pub fn new() -> Self {
        Self {
            board: Board::default(),
            phase: Phase::Idle,
            stats: Stats::load(),
            pending_dice: 0,
            last_roll_text: String::new(),
            cam_shake: 0.0,
            info: String::new(),
            overlay: None,
            flash: Channel::new(0.0),
            confetti_burst: false,
            confetti_clear: false,
        }
    }

    pub fn die_landed(&mut self) {
        if self.phase == Phase::Rolling && self.pending_dice > 0 {
            self.pending_dice -= 1;
            if self.pending_dice == 0 {
                self.phase = Phase::AfterRoll(0.35);
            }
        }
    }

    fn refresh_info(&mut self) {
        self.info = format!(
            "Rolled {}   |   picked {} / {}",
            self.last_roll_text, self.board.selected_sum, self.board.roll_total
        );
    }
}

/// Requests set by the UI / input systems and consumed by `game_system`.
#[derive(Resource, Default)]
pub struct Actions {
    pub roll: bool,
    pub clear: bool,
    pub new_game: bool,
    pub pick: Option<u8>,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Game::new())
            .insert_resource(Actions { new_game: true, ..default() })
            .add_systems(Update, game_system);
    }
}

fn with_tile(tiles: &mut Query<&mut Tile>, n: u8, f: impl FnOnce(&mut Tile)) {
    for mut t in tiles.iter_mut() {
        if t.number == n {
            f(&mut t);
            break;
        }
    }
}

pub fn game_system(
    time: Res<Time>,
    mut game: ResMut<Game>,
    mut act: ResMut<Actions>,
    mut tiles: Query<&mut Tile>,
    mut dice: Query<(&mut Die, &mut Transform, &mut Visibility)>,
) {
    let dt = time.delta_secs();
    game.flash.update(dt);

    if std::mem::take(&mut act.new_game) {
        new_game(&mut game, &mut tiles, &mut dice);
    }
    if std::mem::take(&mut act.roll) && game.phase == Phase::Idle {
        start_roll(&mut game, &mut dice);
    }
    if std::mem::take(&mut act.clear) && game.phase == Phase::Selecting {
        for n in 1..=9u8 {
            if game.board.selected[(n - 1) as usize] {
                with_tile(&mut tiles, n, |t| t.set_selected(false));
            }
        }
        game.board.clear_selection();
        game.refresh_info();
    }
    if let Some(n) = act.pick.take() {
        if game.phase == Phase::Selecting {
            pick(&mut game, &mut tiles, n);
        }
    }

    let phase = game.phase;
    match phase {
        Phase::AfterRoll(t) => {
            let t = t - dt;
            if t > 0.0 {
                game.phase = Phase::AfterRoll(t);
            } else if !game.board.has_move() {
                game_over(&mut game, &mut tiles, false);
            } else {
                game.phase = Phase::Selecting;
                game.refresh_info();
            }
        }
        Phase::Committing(t) => {
            let t = t - dt;
            if t > 0.0 {
                game.phase = Phase::Committing(t);
            } else {
                let mut delay = 0.0f32;
                for n in 1..=9u8 {
                    if game.board.selected[(n - 1) as usize] {
                        with_tile(&mut tiles, n, |tl| tl.flip_down(delay));
                        delay += 0.16;
                    }
                }
                game.board.close_selected();
                game.phase = Phase::Animating(delay + 1.0);
            }
        }
        Phase::Animating(t) => {
            let t = t - dt;
            if t > 0.0 {
                game.phase = Phase::Animating(t);
            } else if game.board.open_sum() == 0 {
                game_over(&mut game, &mut tiles, true);
            } else {
                game.phase = Phase::Idle;
                game.info = "Nice! Roll again".to_string();
            }
        }
        Phase::Ending { win, t } => {
            let t = t - dt;
            if t > 0.0 {
                game.phase = Phase::Ending { win, t };
            } else {
                show_overlay(&mut game, win);
                game.phase = Phase::Over;
            }
        }
        _ => {}
    }
}

fn new_game(
    game: &mut Game,
    tiles: &mut Query<&mut Tile>,
    dice: &mut Query<(&mut Die, &mut Transform, &mut Visibility)>,
) {
    game.board.reset();
    game.phase = Phase::Idle;
    game.pending_dice = 0;
    game.cam_shake = 0.0;
    game.overlay = None;
    game.flash.set(0.0);
    game.confetti_clear = true;
    game.info = "Tap ROLL to throw the dice".to_string();
    for mut t in tiles.iter_mut() {
        let delay = (t.number - 1) as f32 * 0.05;
        t.reset(delay);
    }
    for (mut d, _, mut vis) in dice.iter_mut() {
        d.stop();
        *vis = Visibility::Hidden;
    }
}

fn start_roll(game: &mut Game, dice: &mut Query<(&mut Die, &mut Transform, &mut Visibility)>) {
    let n = if game.board.one_die() { 1 } else { 2 };
    game.phase = Phase::Rolling;
    game.pending_dice = n as u32;
    game.info = "Rolling...".to_string();

    let spots = if n == 1 {
        vec![Vec3::new(0.0, 0.5, 1.8)]
    } else {
        vec![Vec3::new(-1.4, 0.5, 1.9), Vec3::new(1.4, 0.5, 1.7)]
    };
    let mut rng = rand::thread_rng();
    let mut vals = [0u32; 2];
    for (mut die, mut tf, mut vis) in dice.iter_mut() {
        let i = die.index;
        if i < n {
            let v = rng.gen_range(1..=6u32);
            vals[i] = v;
            *vis = Visibility::Visible;
            let to = spots[i] + Vec3::new(rng.gen_range(-0.4..0.4), 0.0, rng.gen_range(-0.3..0.3));
            let side = if i == 0 { -1.0 } else { 1.0 };
            let from = Vec3::new(side * 9.0, 4.0, 5.0);
            die.roll_to(v, from, to, 1.7 + 0.25 * i as f32, &mut tf);
        } else {
            *vis = Visibility::Hidden;
        }
    }
    let total: u32 = vals.iter().sum();
    game.board.roll_total = total;
    game.board.selected_sum = 0;
    game.last_roll_text = if n > 1 {
        format!("{} + {} = {}", vals[0], vals[1], total)
    } else {
        total.to_string()
    };
}

fn pick(game: &mut Game, tiles: &mut Query<&mut Tile>, n: u8) {
    let idx = (n - 1) as usize;
    match game.board.toggle(idx) {
        Toggle::Ignored => return,
        Toggle::Rejected => {
            with_tile(tiles, n, |t| t.shake());
            return;
        }
        Toggle::Selected => with_tile(tiles, n, |t| t.set_selected(true)),
        Toggle::Deselected => with_tile(tiles, n, |t| t.set_selected(false)),
    }
    game.refresh_info();
    if game.board.matched() {
        game.phase = Phase::Committing(0.3);
    }
}

fn game_over(game: &mut Game, tiles: &mut Query<&mut Tile>, win: bool) {
    let score = game.board.open_sum();
    game.stats.games += 1;
    if win {
        game.stats.wins += 1;
    }
    if game.stats.best < 0 || (score as i32) < game.stats.best {
        game.stats.best = score as i32;
    }
    game.stats.save();

    if win {
        game.confetti_burst = true;
        game.info = "SHUT THE BOX!".to_string();
        game.phase = Phase::Ending { win: true, t: 1.2 };
    } else {
        game.info = format!("No combination makes {}", game.board.roll_total);
        game.cam_shake = 0.7;
        game.flash.set(0.4);
        game.flash.go(0.0, 0.8, 0.0, Ease::SineInOut);
        let mut d = 0.0;
        for n in 1..=9u8 {
            if game.board.open[(n - 1) as usize] {
                with_tile(tiles, n, |t| t.mark_lost(d));
                d += 0.06;
            }
        }
        game.phase = Phase::Ending { win: false, t: 1.3 };
    }
}

fn show_overlay(game: &mut Game, win: bool) {
    let s = game.stats;
    game.overlay = Some(if win {
        OverlayInfo {
            title: "BOX SHUT!".into(),
            detail: format!("Perfect score: 0\nBest: 0   Wins: {} / {}", s.wins, s.games),
            color: Color::srgb(1.0, 0.82, 0.35),
        }
    } else {
        OverlayInfo {
            title: "YOU LOST".into(),
            detail: format!(
                "Rolled {} - nothing fits.\nScore: {} (lower is better)\nBest: {}   Wins: {} / {}",
                game.last_roll_text,
                game.board.open_sum(),
                s.best,
                s.wins,
                s.games
            ),
            color: Color::srgb(1.0, 0.38, 0.34),
        }
    });
}
