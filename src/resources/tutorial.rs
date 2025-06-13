// Tooltip sequence for Nexus Core tutorial (Bevy ECS-style)
use bevy::prelude::*;

#[derive(Resource)]
pub struct TutorialState {
    pub steps: Vec<TooltipStep>,
    pub current_step: usize,
    pub highlighted: Option<Entity>,
}

impl Default for TutorialState {
    fn default() -> Self {
        Self {
            steps: get_tutorial_steps(),
            current_step: 0,
            highlighted: None,
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
fn has_entity_with_tag(_world: &World, _tag: &str) -> bool {
    false
}
fn entity_has_flag(_world: &World, _entity: &str, _flag: &str) -> bool {
    false
}
fn entity_produces_resource(_world: &World, _entity: &str) -> bool {
    false
}
fn player_lacks_available_specialists(_world: &World) -> bool {
    false
}
fn population_increased(_world: &World) -> bool {
    false
}
fn happiness_below_threshold(_world: &World, _threshold: f32) -> bool {
    false
}
fn all_services_covered(_world: &World) -> bool {
    false
}
fn tech_tree_opened(_world: &World) -> bool {
    false
}
fn legacy_structure_unlocked(_world: &World) -> bool {
    false
}