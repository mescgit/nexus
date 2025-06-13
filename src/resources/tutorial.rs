// Tooltip sequence for Nexus Core tutorial (Bevy ECS-style)
use bevy::prelude::*;

#[derive(Resource)]
pub struct TutorialState {
    pub steps: Vec<TooltipStep>,
    pub current_step: usize,
}

impl Default for TutorialState {
    fn default() -> Self {
        Self {
            steps: get_tutorial_steps(),
            current_step: 0,
        }
    }
}

pub struct TooltipStep {
    pub trigger: fn(&World) -> bool,
    pub title: &'static str,
    pub content: &'static str,
    pub required_action: Option<fn(&mut World)>,
    pub ui_highlight: Option<&'static str>,
}

pub fn get_tutorial_steps() -> Vec<TooltipStep> {
    vec![
        TooltipStep {
            trigger: |_world| true,
            title: "Welcome to Nexus Core",
            content: "Welcome, Colony Director. Let’s begin by placing your Operations Hub.",
            required_action: None,
            ui_highlight: Some("build_menu.operations_hub"),
        },
        TooltipStep {
            trigger: |world| has_entity_with_tag(world, "operations_hub"),
            title: "Power Online",
            content: "Your Hub is now active and generating power. Time to gather materials.",
            required_action: None,
            ui_highlight: Some("build_menu.extractor"),
        },
        TooltipStep {
            trigger: |world| entity_has_flag(world, "extractor", "needs_power"),
            title: "Power Deficit",
            content: "Not enough power. Build a Power Relay to bring your Extractor online.",
            required_action: None,
            ui_highlight: Some("build_menu.power_relay"),
        },
        TooltipStep {
            trigger: |world| entity_produces_resource(world, "extractor"),
            title: "Resources Flowing",
            content: "You are now producing Ferrocrete and Silica. Monitor your resource panel.",
            required_action: None,
            ui_highlight: Some("ui.resources_panel"),
        },
        TooltipStep {
            trigger: |world| player_lacks_available_specialists(world),
            title: "Need More Citizens",
            content: "You're out of workers. Build housing and food to grow your population.",
            required_action: None,
            ui_highlight: Some("build_menu.basic_dwelling"),
        },
        TooltipStep {
            trigger: |world| has_entity_with_tag(world, "bio_dome"),
            title: "Food Production Started",
            content: "Bio-Dome producing Nutrient Paste. Ensure surplus to enable growth.",
            required_action: None,
            ui_highlight: Some("ui.food_metrics"),
        },
        TooltipStep {
            trigger: |world| population_increased(world),
            title: "Growth Begins",
            content: "Your population is increasing. Assign new Specialists to expand operations.",
            required_action: None,
            ui_highlight: Some("ui.specialists_available"),
        },
        TooltipStep {
            trigger: |world| happiness_below_threshold(world, 70.0),
            title: "Civic Crisis",
            content: "Your citizens are uneasy. Build a Wellness Post and Security Station.",
            required_action: None,
            ui_highlight: Some("build_menu.services"),
        },
        TooltipStep {
            trigger: |world| all_services_covered(world),
            title: "Civics Restored",
            content: "Healthcare and Security restored. Happiness and growth resume.",
            required_action: None,
            ui_highlight: Some("ui.happiness_chart"),
        },
        TooltipStep {
            trigger: |world| has_entity_with_tag(world, "research_institute"),
            title: "Tech Unlocked",
            content: "Research Institute active. Begin unlocking Development Phase 2.",
            required_action: None,
            ui_highlight: Some("ui.tech_tree"),
        },
        TooltipStep {
            trigger: |world| tech_tree_opened(world),
            title: "Research Begins",
            content: "Select a research project to unlock new buildings and capabilities.",
            required_action: None,
            ui_highlight: Some("tech_tree.initial_node"),
        },
        TooltipStep {
            trigger: |world| legacy_structure_unlocked(world),
            title: "Legacy Awaits",
            content: "Your colony is thriving. Begin preparation for the Genesis Monument.",
            required_action: None,
            ui_highlight: Some("ui.legacy_panel"),
        },
    ]
}

// Placeholder condition helpers
use crate::game_state::{GameState, ResourceType, ServiceType};

fn has_entity_with_tag(world: &World, tag: &str) -> bool {
    let state = world.resource::<GameState>();
    match tag {
        // Treat the Operations Hub as the Administrative Spire in this prototype
        "operations_hub" => state.administrative_spire.is_some(),
        "extractor" => !state.extractors.is_empty(),
        "bio_dome" => !state.bio_domes.is_empty(),
        "power_relay" => !state.power_relays.is_empty(),
        "storage_silo" => !state.storage_silos.is_empty(),
        "research_institute" => !state.research_institutes.is_empty(),
        _ => false,
    }
}

fn entity_has_flag(world: &World, entity: &str, flag: &str) -> bool {
    let state = world.resource::<GameState>();
    match (entity, flag) {
        ("extractor", "needs_power") => {
            !state.extractors.is_empty()
                && state.total_generated_power < state.total_consumed_power
        }
        _ => false,
    }
}

fn entity_produces_resource(world: &World, entity: &str) -> bool {
    let state = world.resource::<GameState>();
    match entity {
        "extractor" => {
            if state.extractors.is_empty() {
                return false;
            }
            if let Some(amount) = state.current_resources.get(&ResourceType::FerrocreteOre) {
                // Initial state grants 200 units; any increase implies production
                *amount > 200.0
            } else {
                false
            }
        }
        _ => false,
    }
}

fn player_lacks_available_specialists(world: &World) -> bool {
    let state = world.resource::<GameState>();
    state.assigned_specialists_total >= state.total_specialist_slots && state.total_specialist_slots > 0
}

fn population_increased(world: &World) -> bool {
    let state = world.resource::<GameState>();
    state.total_inhabitants > 5
}

fn happiness_below_threshold(world: &World, threshold: f32) -> bool {
    let state = world.resource::<GameState>();
    state.colony_happiness < threshold
}

fn all_services_covered(world: &World) -> bool {
    let coverage = world.resource::<crate::game_state::ServiceCoverage>();
    let service_types = [
        ServiceType::Wellness,
        ServiceType::Security,
        ServiceType::Education,
        ServiceType::Recreation,
        ServiceType::Spiritual,
    ];

    service_types
        .iter()
        .all(|t| coverage.coverage.get(t).copied().unwrap_or(0.0) >= 1.0)
}

fn tech_tree_opened(world: &World) -> bool {
    let state = world.resource::<GameState>();
    state.research_progress.is_some()
}

fn legacy_structure_unlocked(world: &World) -> bool {
    let state = world.resource::<GameState>();
    state.legacy_structure.is_some()
}
