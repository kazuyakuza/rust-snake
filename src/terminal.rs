//! Terminal UI layer: rendering, arrow-key input, and terminal lifecycle.
//! Gameplay rules remain in `game`; this layer only renders state, maps keys,
//! and toggles raw mode / alternate screen / cursor visibility.

pub mod input;
pub mod lifecycle;
pub mod renderer;
