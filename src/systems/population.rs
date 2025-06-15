use bevy::prelude::*;
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
        let growth_rate = population.count as f32 * 0.01 * happiness_factor;
        game_state.population_growth_progress += growth_rate;
        let added = game_state.population_growth_progress.floor() as u32;
        if added > 0 {
            population.count += added;
            game_state.population_growth_progress -= added as f32;
            population.count = population
                .count
                .min(game_state.available_housing_capacity);
        }
    }

    game_state.total_inhabitants = population.count;
}