//! Terminal UI layer: rendering, arrow-key input, timed playing loop, and terminal
//! lifecycle. Gameplay rules remain in `game`; this layer only renders state, maps
//! keys, ticks the loop, and toggles raw mode / alternate screen / cursor visibility.

pub mod game_loop;
pub mod input;
pub mod lifecycle;
pub mod renderer;
