use bevy::prelude::*;
use rand::Rng;

use crate::{
    game_state::{GameState, ResourceType},
    resources::population::PopulationResource,
};

pub fn population_growth_system(
    mut game_state: ResMut<GameState>,
    mut population: ResMut<PopulationResource>,
) {
    population.count = game_state.total_inhabitants;

    let has_housing = population.count < game_state.available_housing_capacity;
    let food_amount = *game_state
        .current_resources
        .get(&ResourceType::NutrientPaste)
        .unwrap_or(&0.0);
    let has_food = food_amount > 0.0;

    let happiness_factor = (game_state.colony_happiness - 50.0) / 50.0;
    if has_housing && has_food && happiness_factor > 0.0 {
        let growth_chance_per_sec = happiness_factor * 0.1;
        if rand::thread_rng().gen::<f32>() < growth_chance_per_sec {
            population.count += 1;
        }
    }

    game_state.total_inhabitants = population.count;
}
