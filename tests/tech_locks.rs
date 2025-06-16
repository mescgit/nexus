use nexus::game_state::{self, GameState, Tech, ServiceType, ZoneType};

#[test]
fn add_habitation_structure_locked_tech() {
    let mut state = GameState::default();
    let initial_credits = state.credits;
    let initial_len = state.habitation_structures.len();
    let initial_notifs = state.notifications.len();

    game_state::add_habitation_structure(&mut state, 2, None);

    assert_eq!(state.credits, initial_credits);
    assert_eq!(state.habitation_structures.len(), initial_len);
    assert_eq!(state.notifications.len(), initial_notifs + 1);
    let msg = &state.notifications.front().unwrap().message;
    assert!(msg.contains("Technology") && msg.contains("Arcology"));
}

#[test]
fn add_service_building_locked_tech() {
    let mut state = GameState::default();
    let initial_credits = state.credits;
    let initial_len = state.service_buildings.len();
    let initial_notifs = state.notifications.len();

    game_state::add_service_building(&mut state, ServiceType::Security, 1, None);

    assert_eq!(state.credits, initial_credits);
    assert_eq!(state.service_buildings.len(), initial_len);
    assert_eq!(state.notifications.len(), initial_notifs + 1);
    let msg = &state.notifications.front().unwrap().message;
    assert!(msg.contains("Technology") && msg.contains("Precinct"));
}

#[test]
fn add_zone_locked_tech() {
    let mut state = GameState::default();
    let initial_credits = state.credits;
    let initial_len = state.zones.len();
    let initial_notifs = state.notifications.len();

    game_state::add_zone(&mut state, ZoneType::Commercial, 1);

    assert_eq!(state.credits, initial_credits);
    assert_eq!(state.zones.len(), initial_len);
    assert_eq!(state.notifications.len(), initial_notifs + 1);
    let msg = &state.notifications.front().unwrap().message;
    assert!(msg.contains("Technology") && msg.contains("Shopping Plaza"));
}
