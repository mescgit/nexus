use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game_state::{GameState, ServiceCoverage, ServiceType};
use crate::resources::population::PopulationResource; // not used but show typical cross refs

#[derive(Resource, Serialize, Deserialize, Clone)]
pub struct HappinessResource {
    pub score: f32,
}

impl Default for HappinessResource {
    fn default() -> Self {
        HappinessResource { score: 50.0 }
    }
}

pub fn calculate_colony_happiness(game_state: &mut GameState, coverage: &ServiceCoverage) {
    let mut happiness_score = 50.0;

    if game_state.simulated_has_sufficient_nutrient_paste {
        happiness_score += 10.0;
    } else {
        happiness_score -= 25.0;
    }

    if game_state.total_inhabitants > game_state.available_housing_capacity {
        let homeless = game_state.total_inhabitants - game_state.available_housing_capacity;
        happiness_score -= (homeless as f32) * 2.0;
    } else if game_state.available_housing_capacity > 0 && game_state.total_inhabitants > 0 {
        let occupancy_ratio = game_state.total_inhabitants as f32 / game_state.available_housing_capacity as f32;
        if occupancy_ratio <= 0.9 {
            happiness_score += 5.0;
        } else if occupancy_ratio < 1.0 {
            happiness_score += 2.0;
        }
    }

    if let Some(structure) = &game_state.legacy_structure {
        if let Some(tier) = structure.available_tiers.get(structure.current_tier_index) {
            happiness_score += tier.happiness_bonus;
        }
    }
    happiness_score += (game_state.civic_index as f32 / 10.0).min(5.0);

    let service_types = [
        ServiceType::Wellness,
        ServiceType::Security,
        ServiceType::Education,
        ServiceType::Recreation,
        ServiceType::Spiritual,
    ];

    for service_type in service_types {
        if let Some(ratio) = coverage.coverage.get(&service_type) {
            if *ratio >= 1.0 {
                happiness_score += 5.0;
            } else {
                happiness_score -= (1.0 - ratio) * 10.0;
            }
        }
    }

    game_state.colony_happiness = happiness_score.clamp(0.0, 100.0);
}

pub fn happiness_system(
    mut game_state: ResMut<GameState>,
    mut happiness: ResMut<HappinessResource>,
    coverage: Res<ServiceCoverage>,
) {
    calculate_colony_happiness(&mut game_state, &coverage);
    happiness.score = game_state.colony_happiness;
}