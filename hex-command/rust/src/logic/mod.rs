//! Battle rules and campaign transitions, independent of transport and policy.
// Speculative drafts update mechanics without building presentation-only strings.
macro_rules! game_log {
    ($game:expr, format!($($arg:tt)* ) $(,)?) => {
        if $game.presentation {
            use std::fmt::Write as _;
            let mut buffer = $game.take_log_buffer();
            write!(&mut buffer, $($arg)*).expect("writing a log buffer");
            $game.record(buffer);
        }
    };
    ($game:expr, $message:expr $(,)?) => {
        game_log!($game, format!("{}", $message));
    };
}
#[cfg(not(target_arch = "wasm32"))]
macro_rules! game_action {
    ($game:expr, $action:expr $(,)?) => {
        if $game.recordings.is_some() {
            $game.action($action);
        }
    };
}
pub(crate) mod action;
pub(crate) mod battle;
mod combat;
mod commands;
pub(crate) use action::Action;
pub(crate) mod damage;
pub(crate) mod exchange;
mod forks;
#[cfg(not(target_arch = "wasm32"))]
mod indices;
#[cfg(any(test, feature = "wasm"))]
mod inspection;
mod movement;
#[cfg(feature = "wasm")]
pub(crate) mod playback;
mod restore;
mod storage;
mod turns;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use restore::validate_state_geometry;
pub use restore::{StateError, validate_state};
pub mod campaign;
pub use crate::data::{Command, CommandError};
pub(crate) use battle::Game;

pub(crate) mod geometry;

// These maps are small publication/campaign tables. Incremental insertion avoids
// compiling a separate bulk stable-sort pipeline for each key/value pair.
fn collect_map<K: Ord, V>(
    entries: impl IntoIterator<Item = (K, V)>,
) -> std::collections::BTreeMap<K, V> {
    let mut map = std::collections::BTreeMap::new();
    map.extend(entries);
    map
}
