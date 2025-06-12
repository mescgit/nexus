use bevy::prelude::*;

use crate::game_state::{GameState, ServiceCoverage, ServiceType};

pub fn service_coverage_system(
    game_state: Res<GameState>,
    mut coverage: ResMut<ServiceCoverage>,
) {
    coverage.coverage.clear();
    let demand = game_state.total_inhabitants;
    let service_types = [
        ServiceType::Wellness,
        ServiceType::Security,
        ServiceType::Education,
        ServiceType::Recreation,
        ServiceType::Spiritual,
    ];

    for service_type in service_types {
        if demand == 0 {
            coverage.coverage.insert(service_type, 1.0);
            continue;
        }

        let mut supply = 0;
        for building in &game_state.service_buildings {
            if building.service_type == service_type && building.is_active {
                if let Some(tier) = building.available_tiers.get(building.current_tier_index) {
                    if building.assigned_specialists >= tier.specialist_requirement {
                        let in_range = if let Some(b_pos) = building.position {
                            game_state.habitation_structures.iter().any(|hab| {
                                if let Some(h_pos) = hab.position {
                                    let dx = b_pos.0 - h_pos.0;
                                    let dy = b_pos.1 - h_pos.1;
                                    let dist2 = dx * dx + dy * dy;
                                    dist2 <= tier.service_radius * tier.service_radius
                                } else {
                                    false
                                }
                            })
                        } else {
                            true
                        };

                        if in_range {
                            supply += tier.service_capacity;
                        }
                    }
                }
            }
        }

        let ratio = (supply as f32 / demand as f32).min(1.0);
        coverage.coverage.insert(service_type, ratio);
    }
}