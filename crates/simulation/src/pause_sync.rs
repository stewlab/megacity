//! P0-09: Unify pause authority — keep `AppState` in sync with `GameClock.paused`.
//!
//! Two independent pause mechanisms exist:
//! - `GameClock.paused` — toggled by toolbar speed controls and keybinds
//! - `AppState::Paused` — toggled by the ESC pause menu
//!
//! This module adds a sync system that watches `GameClock.paused` and transitions
//! `AppState` accordingly, so both mechanisms always agree.  When `GameClock.paused`
//! is set to `true` while `AppState` is `Playing`, we transition to `Paused`.
//! When `GameClock.paused` is set to `false` while `AppState` is `Paused`, we
//! transition back to `Playing`.
//!
//! Exception: a clock pause held by the tutorial (`TutorialState::paused_by_tutorial`)
//! is not mirrored to `AppState` — the tutorial pauses time while keeping the game
//! in `Playing` so its instruction window stays visible.
//!
//! The sync is one-directional: `GameClock.paused` is the source of truth.
//! Code that transitions `AppState` (e.g. the pause menu) must also set
//! `GameClock.paused` — which it already does.

use bevy::prelude::*;

use crate::app_state::AppState;
use crate::time_of_day::GameClock;
use crate::tutorial::TutorialState;

/// Keeps `AppState` in sync with `GameClock.paused`.
///
/// Runs every frame (not gated by `AppState::Playing`) so it can detect
/// when the toolbar or speed keybinds pause/unpause via `GameClock` and
/// mirror the change to `AppState`.
///
/// The tutorial also pauses the clock on its instruction steps while keeping
/// the game interactive. That is not a user pause: while the tutorial holds
/// the clock (`TutorialState::paused_by_tutorial`), `AppState` must stay in
/// `Playing` so the tutorial window (gated on `Playing`) remains visible.
/// Mirroring a tutorial-held pause would hide the tutorial UI and, because
/// the tutorial re-pauses the clock every frame, instantly re-enter
/// `Paused` after every resume — an inescapable pause loop on every new game.
fn sync_pause_state(
    clock: Res<GameClock>,
    tutorial: Res<TutorialState>,
    app_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let tutorial_holds_pause = tutorial.paused_by_tutorial;
    match app_state.get() {
        AppState::Playing if clock.paused && !tutorial_holds_pause => {
            next_state.set(AppState::Paused);
        }
        AppState::Paused if !clock.paused => {
            next_state.set(AppState::Playing);
        }
        _ => {}
    }
}

pub struct PauseSyncPlugin;

impl Plugin for PauseSyncPlugin {
    fn build(&self, app: &mut App) {
        // Run in Update without any state gate — this system must observe
        // transitions *from* any gameplay state.
        app.add_systems(Update, sync_pause_state);
    }
}
