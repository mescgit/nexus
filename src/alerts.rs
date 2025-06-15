use bevy::prelude::*;
use crate::game_state::{
    self, GameState, ResourceType, BASE_STORAGE_CAPACITY, STORAGE_SILO_CAPACITY,
};

#[derive(Resource, Default)]
pub struct AlertState {
    power: bool,
    food: bool,
    unrest: bool,
    cap: bool,
}

pub struct AlertPlugin;

impl Plugin for AlertPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AlertState>()
            .add_systems(Update, alert_system);
    }
}

fn alert_system(mut alert: ResMut<AlertState>, mut game_state: ResMut<GameState>, time: Res<Time>) {
    let now = time.elapsed_seconds_f64();
    let net_power = game_state.total_generated_power - game_state.total_consumed_power;
    if net_power < 0.0 && !alert.power {
        game_state::add_notification(&mut game_state.notifications, "ALERT: Power deficit detected.".to_string(), now);
        alert.power = true;
    } else if net_power >= 0.0 && alert.power {
        game_state::add_notification(&mut game_state.notifications, "Power levels stabilized.".to_string(), now);
        alert.power = false;
    }

    let food = *game_state.current_resources.get(&ResourceType::NutrientPaste).unwrap_or(&0.0);
    if food < 10.0 && !alert.food {
        game_state::add_notification(&mut game_state.notifications, "ALERT: Food shortage.".to_string(), now);
        alert.food = true;
    } else if food >= 10.0 && alert.food {
        game_state::add_notification(&mut game_state.notifications, "Food supply restored.".to_string(), now);
        alert.food = false;
    }

    if game_state.colony_happiness < 30.0 && !alert.unrest {
        game_state::add_notification(&mut game_state.notifications, "ALERT: Civic unrest rising.".to_string(), now);
        alert.unrest = true;
    } else if game_state.colony_happiness >= 30.0 && alert.unrest {
        game_state::add_notification(&mut game_state.notifications, "Civic order restored.".to_string(), now);
        alert.unrest = false;
    }

    let capacity = BASE_STORAGE_CAPACITY + game_state.storage_silos.len() as f32 * STORAGE_SILO_CAPACITY;
    let nearing_cap = game_state
        .current_resources
        .values()
        .any(|&v| v >= capacity * 0.9);
    if nearing_cap && !alert.cap {
        game_state::add_notification(&mut game_state.notifications, "Storage capacity nearly full.".to_string(), now);
        alert.cap = true;
    } else if !nearing_cap && alert.cap {
        alert.cap = false;
    }
}