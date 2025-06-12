// Tooltip sequence for Nexus Core tutorial (Bevy ECS-style)
use bevy::prelude::*;
use crate::{
    game_state::{GameState, ServiceType},
    CurrentApp, AppType,
};

#[derive(Resource)]
pub struct TutorialState {
    pub current_step: usize,
    pub completed_steps: Vec<bool>,
    pub active_tooltip: Option<Entity>,
}

impl Default for TutorialState {
    fn default() -> Self {
        let steps = get_tutorial_steps();
        Self {
            current_step: 0,
            completed_steps: vec![false; steps.len()],
            active_tooltip: None,
        }
    }
}

pub struct TooltipStep {
    pub trigger: fn(&GameState, &CurrentApp) -> bool,
    pub title: &'static str,
    pub content: &'static str,
    pub required_action: Option<fn(&mut World)>,
    pub ui_highlight: Option<&'static str>,
}

pub fn get_tutorial_steps() -> Vec<TooltipStep> {
    vec![
        TooltipStep {
            trigger: |_gs, _app| true,
            title: "Welcome to Nexus Core",
            content: "Welcome, Colony Director. Let’s begin by placing your Operations Hub.",
            required_action: None,
            ui_highlight: Some("build_menu.operations_hub"),
        },
        TooltipStep {
            trigger: |gs, _app| has_entity_with_tag(gs, "operations_hub"),
            title: "Power Online",
            content: "Your Hub is now active and generating power. Time to gather materials.",
            required_action: None,
            ui_highlight: Some("build_menu.extractor"),
        },
        TooltipStep {
            trigger: |gs, _app| entity_has_flag(gs, "extractor", "needs_power"),
            title: "Power Deficit",
            content: "Not enough power. Build a Power Relay to bring your Extractor online.",
            required_action: None,
            ui_highlight: Some("build_menu.power_relay"),
        },
        TooltipStep {
            trigger: |gs, _app| entity_produces_resource(gs, "extractor"),
            title: "Resources Flowing",
            content: "You are now producing Ferrocrete and Silica. Monitor your resource panel.",
            required_action: None,
            ui_highlight: Some("ui.resources_panel"),
        },
        TooltipStep {
            trigger: |gs, _app| player_lacks_available_specialists(gs),
            title: "Need More Citizens",
            content: "You're out of workers. Build housing and food to grow your population.",
            required_action: None,
            ui_highlight: Some("build_menu.basic_dwelling"),
        },
        TooltipStep {
            trigger: |gs, _app| has_entity_with_tag(gs, "bio_dome"),
            title: "Food Production Started",
            content: "Bio-Dome producing Nutrient Paste. Ensure surplus to enable growth.",
            required_action: None,
            ui_highlight: Some("ui.food_metrics"),
        },
        TooltipStep {
            trigger: |gs, _app| population_increased(gs),
            title: "Growth Begins",
            content: "Your population is increasing. Assign new Specialists to expand operations.",
            required_action: None,
            ui_highlight: Some("ui.specialists_available"),
        },
        TooltipStep {
            trigger: |gs, _app| happiness_below_threshold(gs, 70.0),
            title: "Civic Crisis",
            content: "Your citizens are uneasy. Build a Wellness Post and Security Station.",
            required_action: None,
            ui_highlight: Some("build_menu.services"),
        },
        TooltipStep {
            trigger: |gs, _app| all_services_covered(gs),
            title: "Civics Restored",
            content: "Healthcare and Security restored. Happiness and growth resume.",
            required_action: None,
            ui_highlight: Some("ui.happiness_chart"),
        },
        TooltipStep {
            trigger: |gs, _app| has_entity_with_tag(gs, "research_institute"),
            title: "Tech Unlocked",
            content: "Research Institute active. Begin unlocking Development Phase 2.",
            required_action: None,
            ui_highlight: Some("ui.tech_tree"),
        },
        TooltipStep {
            trigger: |_gs, app| tech_tree_opened(app),
            title: "Research Begins",
            content: "Select a research project to unlock new buildings and capabilities.",
            required_action: None,
            ui_highlight: Some("tech_tree.initial_node"),
        },
        TooltipStep {
            trigger: |gs, _app| legacy_structure_unlocked(gs),
            title: "Legacy Awaits",
            content: "Your colony is thriving. Begin preparation for the Genesis Monument.",
            required_action: None,
            ui_highlight: Some("ui.legacy_panel"),
        },
    ]
}

// Placeholder condition helpers
fn has_entity_with_tag(gs: &GameState, tag: &str) -> bool {
    match tag {
        "operations_hub" => gs.administrative_spire.is_some(),
        "extractor" => !gs.extractors.is_empty(),
        "bio_dome" => !gs.bio_domes.is_empty(),
        "research_institute" => !gs.research_institutes.is_empty(),
        _ => false,
    }
}

fn entity_has_flag(gs: &GameState, entity: &str, flag: &str) -> bool {
    if entity == "extractor" && flag == "needs_power" {
        !gs.extractors.is_empty()
            && (gs.total_generated_power - gs.total_consumed_power) < 0.0
    } else {
        false
    }
}

fn entity_produces_resource(gs: &GameState, entity: &str) -> bool {
    if entity == "extractor" {
        gs.extractors.iter().any(|e| e.is_staffed)
    } else {
        false
    }
}

fn player_lacks_available_specialists(gs: &GameState) -> bool {
    gs.assigned_workforce >= gs.total_inhabitants
}

fn population_increased(gs: &GameState) -> bool {
    gs.total_inhabitants > 5
}

fn happiness_below_threshold(gs: &GameState, threshold: f32) -> bool {
    gs.colony_happiness < threshold
}

fn all_services_covered(gs: &GameState) -> bool {
    [ServiceType::Wellness, ServiceType::Security]
        .iter()
        .all(|t| gs.service_buildings.iter().any(|b| b.service_type == *t))
}

fn tech_tree_opened(app: &CurrentApp) -> bool {
    app.0 == AppType::Research
}

fn legacy_structure_unlocked(gs: &GameState) -> bool {
    gs.legacy_structure.is_some()
}

// --- Tutorial Plugin Implementation ---

#[derive(Component)]
struct TutorialTooltip;

#[derive(Component)]
struct TutorialOkButton;

pub struct TutorialPlugin;

impl Plugin for TutorialPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TutorialState>()
            .add_systems(Update, (check_tutorial_triggers_system, tooltip_button_system));
    }
}

fn check_tutorial_triggers_system(
    mut commands: Commands,
    mut state: ResMut<TutorialState>,
    game_state: Res<GameState>,
    current_app: Res<CurrentApp>,
) {
    let steps = get_tutorial_steps();
    if state.current_step >= steps.len() {
        return;
    }

    let step = &steps[state.current_step];
    if (step.trigger)(&game_state, &current_app) && state.active_tooltip.is_none() {
        let tooltip = spawn_tooltip(&mut commands, step);
        state.active_tooltip = Some(tooltip);
        state.completed_steps[state.current_step] = true;
    }
    if state.active_tooltip.is_some() {
        // update timestamp maybe? ignoring for now
    } else if state.completed_steps[state.current_step] {
        state.current_step += 1;
    }
}

fn spawn_tooltip(commands: &mut Commands, step: &TooltipStep) -> Entity {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(20.0),
                    left: Val::Px(20.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: Color::rgba(0.0, 0.0, 0.2, 0.8).into(),
                border_color: Color::CYAN.into(),
                ..default()
            },
            TutorialTooltip,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                step.title,
                TextStyle {
                    font_size: 18.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
            parent.spawn(TextBundle::from_section(
                step.content,
                TextStyle {
                    font_size: 14.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
            if let Some(highlight) = step.ui_highlight {
                parent.spawn(TextBundle::from_section(
                    format!("Hint: {}", highlight),
                    TextStyle {
                        font_size: 12.0,
                        color: Color::YELLOW,
                        ..default()
                    },
                ));
            }
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            margin: UiRect::top(Val::Px(5.0)),
                            padding: UiRect::all(Val::Px(5.0)),
                            ..default()
                        },
                        background_color: Color::DARK_GRAY.into(),
                        ..default()
                    },
                    TutorialOkButton,
                ))
                .with_children(|btn| {
                    btn.spawn(TextBundle::from_section(
                        "OK",
                        TextStyle {
                            font_size: 14.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ));
                });
        })
        .id()
}

fn tooltip_button_system(
    mut commands: Commands,
    mut interaction_q: Query<(&Interaction, Entity), (Changed<Interaction>, With<TutorialOkButton>)>,
    mut state: ResMut<TutorialState>,
) {
    for (interaction, _entity) in &mut interaction_q {
        if *interaction == Interaction::Pressed {
            if let Some(root) = state.active_tooltip.take() {
                commands.entity(root).despawn_recursive();
            }
            state.current_step += 1;
        }
    }
}
