use bevy::prelude::*;
use crate::game_state::GameState; // For ResMut<GameState>
// Assuming add_extractor is a free function in game_state.rs
// and takes GameState and an Option for position.

pub fn minimal_construction_test_system(mut game_state: ResMut<GameState>) {
    // Attempt to call one of the add_X functions using its fully qualified path.
    // Position is None for this test.
    crate::game_state::add_extractor(&mut game_state, None);
    bevy::log::info!("minimal_construction_test_system: Attempted to call add_extractor.");
}

// All other systems, components, and the build function are commented out for this test.
