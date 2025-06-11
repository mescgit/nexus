// src/main.rs

use bevy::prelude::*;
// REMOVED: `use bevy_prototype_lyon::prelude::*;`
use game_state::{
    BasicDwelling, BioDome, BuildingType, ColonyStats, Extractor, GameState, GraphData, PowerRelay,
    ResearchInstitute, ResourceType, SecurityStation, StorageSilo, Tech, WellnessPost,
    AdministrativeSpireTier, // Added for admin_spire_button_system
};

mod game_state;

// Define the trait
trait GraphableFn: Fn(&ColonyStats) -> f32 {}

// Implement the trait for all closures that fit the signature
impl<F: Fn(&ColonyStats) -> f32> GraphableFn for F {}

// --- UI Marker Components ---
#[derive(Component)]
struct PowerText;
#[derive(Component)]
struct ResourceText(ResourceType);
#[derive(Component)]
struct BuildButton(BuildingType);
#[derive(Component)]
struct ResearchButton(Tech);
#[derive(Component)]
struct MessageText;
#[derive(Component)]
struct ColonyStatText(StatType);
#[derive(Component)]
struct CreditsText; 
#[derive(Component)]
struct PopulationText; 
#[derive(Component)]
struct CoreResourceText(ResourceType); 
#[derive(Component)]
struct ColonyHappinessText; 
#[derive(Component)]
struct ConstructSpireButton;
#[derive(Component)]
struct UpgradeSpireButton;
#[derive(Component)]
struct BuildHabitationButton(usize); 
#[derive(Component)]
struct BuildServiceBuildingButton {
    service_type: game_state::ServiceType, 
    tier_index: usize,
}
#[derive(Component)]
struct BuildZoneButton {
    zone_type: game_state::ZoneType, 
    tier_index: usize,
}
#[derive(Component)]
struct GraphArea; 

// --- App Panel Marker Components ---
#[derive(Component)]
struct DashboardPanel;
#[derive(Component)]
struct NotificationsPanel;
#[derive(Component)]
struct AnalyticsGraphPanel;
#[derive(Component)]
struct ConstructionPanel;
#[derive(Component)]
struct ColonyStatusPanel;
#[derive(Component)]
struct ResearchPanel;

#[derive(Component)] 
struct AppDrawerButton(AppType);

#[derive(Component)] // New Marker Component
struct AdminSpireStatusText;


use crate::game_state::{BuildingType as GameBuildingType, DevelopmentPhase, ALL_BUILDING_TYPES}; 
use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub struct BuildingMetadata {
    pub name: &'static str,
    pub category: ConstructionCategory,
    pub required_tech: Option<Tech>,
    pub required_dp: Option<DevelopmentPhase>, 
}

fn get_building_metadata() -> HashMap<GameBuildingType, BuildingMetadata> {
    let mut meta = HashMap::new();
    meta.insert(GameBuildingType::Extractor, BuildingMetadata { name: "Extractor", category: ConstructionCategory::Operations, required_tech: None, required_dp: None });
    meta.insert(GameBuildingType::BioDome, BuildingMetadata { name: "Bio-Dome", category: ConstructionCategory::Operations, required_tech: None, required_dp: None });
    meta.insert(GameBuildingType::PowerRelay, BuildingMetadata { name: "Power Relay", category: ConstructionCategory::Operations, required_tech: None, required_dp: None });
    meta.insert(GameBuildingType::StorageSilo, BuildingMetadata { name: "Storage Silo", category: ConstructionCategory::Operations, required_tech: Some(Tech::BasicConstructionProtocols), required_dp: None });
    meta.insert(GameBuildingType::ResearchInstitute, BuildingMetadata { name: "Research Institute", category: ConstructionCategory::Operations, required_tech: Some(Tech::BasicConstructionProtocols), required_dp: None });
    meta.insert(GameBuildingType::Fabricator, BuildingMetadata { name: "Fabricator", category: ConstructionCategory::Operations, required_tech: Some(Tech::BasicConstructionProtocols), required_dp: None });
    meta.insert(GameBuildingType::ProcessingPlant, BuildingMetadata { name: "Processing Plant", category: ConstructionCategory::Operations, required_tech: Some(Tech::BasicConstructionProtocols), required_dp: None });
    meta.insert(GameBuildingType::BasicDwelling, BuildingMetadata { name: "Basic Dwelling", category: ConstructionCategory::Habitation, required_tech: Some(Tech::BasicConstructionProtocols), required_dp: None });
    meta.insert(GameBuildingType::WellnessPost, BuildingMetadata { name: "Wellness Post", category: ConstructionCategory::Habitation, required_tech: Some(Tech::BasicConstructionProtocols), required_dp: None });
    meta.insert(GameBuildingType::SecurityStation, BuildingMetadata { name: "Security Station", category: ConstructionCategory::Habitation, required_tech: Some(Tech::BasicConstructionProtocols), required_dp: None });
    meta
}

#[derive(Resource, Default, Debug)] 
pub struct SelectedBuilding(pub Option<GameBuildingType>);

#[derive(Component)]
struct ConfirmBuildButton(GameBuildingType);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConstructionCategory {
    #[default]
    Operations, 
    Habitation, 
    Legacy,     
}

#[derive(Resource, Default)]
pub struct CurrentConstructionCategory(pub ConstructionCategory);

#[derive(Component)]
struct ConstructionCategoryTab(ConstructionCategory);

#[derive(Component)]
struct ConstructionItemListPanel; 

#[derive(Component)]
struct ConstructionItemButton(GameBuildingType); 

#[derive(Component)]
struct ConstructionItemDetailsPanel; 

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppType {
    #[default]
    Dashboard,
    Construction,
    ColonyStatus,
    Research,
}

#[derive(Resource, Default)]
pub struct CurrentApp(pub AppType);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum StatType {
    Housing,
    Jobs,
    Health,
    Police,
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

const HOUSING_COLOR: Color = Color::rgb(0.2, 0.7, 1.0);
const JOBS_COLOR: Color = Color::rgb(1.0, 0.7, 0.2);
const HEALTH_COLOR: Color = Color::rgb(0.2, 1.0, 0.7);
const POLICE_COLOR: Color = Color::rgb(1.0, 0.2, 0.2);

const CREDITS_COLOR: Color = Color::rgb(0.9, 0.8, 0.2); 
const NET_POWER_COLOR: Color = Color::rgb(0.4, 0.6, 1.0); 
const HAPPINESS_COLOR: Color = Color::rgb(1.0, 0.5, 0.8); 
const NUTRIENT_PASTE_COLOR: Color = Color::rgb(0.5, 0.9, 0.3); 

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Nexus Core: Colony Manager".into(),
                    ..default()
                }),
                ..default()
            }),
            game_state::GameLogicPlugin,
            UiPlugin,
        ))
        .insert_resource(Time::<Fixed>::from_seconds(1.0))
        .run();
}

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MessageLog>()
            .init_resource::<CurrentApp>() 
            .init_resource::<CurrentConstructionCategory>() 
            .init_resource::<SelectedBuilding>() 
            .add_systems(Startup, setup_ui)
            .add_systems(Update, (
                app_drawer_button_system,
                manage_app_panels_visibility, 
                update_dashboard_notifications_system.after(manage_app_panels_visibility),
                construction_category_tab_system, 
                update_construction_list_system.after(construction_category_tab_system),
                construction_item_interaction_system.after(update_construction_list_system),
                update_construction_details_panel_system.after(construction_item_interaction_system),
                construction_interaction_system.after(update_construction_details_panel_system), 
                update_text_display,
                draw_graph_gizmos, 
                update_admin_spire_status_system, // Added new system
                admin_spire_button_system, // Ensure this is still here if it has UI elements in ConstructionPanel
            ));
    }
}

#[derive(Resource, Default)]
struct MessageLog { message: String } 

fn setup_ui(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(NodeBundle {
        style: Style { width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, justify_content: JustifyContent::SpaceBetween, ..default() }, ..default()
    }).with_children(|parent| {
        parent.spawn(NodeBundle {
            style: Style { width: Val::Percent(100.0), padding: UiRect::all(Val::Px(10.0)), align_items: AlignItems::Center, flex_wrap: FlexWrap::Wrap, ..default() },
            background_color: Color::DARK_GRAY.into(), ..default()
        }).with_children(|parent| {
            parent.spawn((TextBundle::from_section("Cr. 0", TextStyle { font_size: 18.0, color: Color::GOLD, ..default() }).with_style(Style { margin: UiRect { right: Val::Px(15.0), ..default() }, ..default() }), CreditsText));
            parent.spawn((TextBundle::from_section("Power - Net: +0 | Stored: 0/0", TextStyle { font_size: 18.0, color: Color::CYAN, ..default() }).with_style(Style { margin: UiRect { right: Val::Px(15.0), ..default() }, ..default() }), PowerText));
            parent.spawn((TextBundle::from_section("Inhabitants: 0 / 0", TextStyle { font_size: 18.0, color: Color::WHITE, ..default() }).with_style(Style { margin: UiRect { right: Val::Px(15.0), ..default() }, ..default() }), PopulationText));
            let core_resources_to_display = [ResourceType::NutrientPaste, ResourceType::FerrocreteOre, ResourceType::CuprumDeposits];
            for resource_type in core_resources_to_display {
                parent.spawn((TextBundle::from_section(format!("{:?}: 0", resource_type), TextStyle { font_size: 18.0, color: Color::GRAY, ..default() }).with_style(Style { margin: UiRect { right: Val::Px(10.0), ..default() }, ..default() }), CoreResourceText(resource_type)));
            }
            parent.spawn((TextBundle::from_section("😐 0%", TextStyle { font_size: 18.0, color: Color::YELLOW, ..default() }).with_style(Style { margin: UiRect { left: Val::Px(15.0), right: Val::Px(0.0), top: Val::Px(0.0), bottom: Val::Px(0.0) }, ..default() }), ColonyHappinessText));
        });

        parent.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0), 
                flex_direction: FlexDirection::Row,
                ..default()
            },
            ..default()
        }).with_children(|parent| {
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(15.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                background_color: Color::rgb(0.1, 0.1, 0.1).into(),
                ..default()
            }).with_children(|parent| {
                let app_buttons = [AppType::Dashboard, AppType::Construction, AppType::ColonyStatus, AppType::Research];
                for app_type in app_buttons {
                    parent.spawn((
                        ButtonBundle {
                            style: Style {
                                width: Val::Percent(95.0),
                                padding: UiRect::all(Val::Px(10.0)),
                                margin: UiRect::all(Val::Px(3.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            background_color: NORMAL_BUTTON.into(),
                            ..default()
                        },
                        AppDrawerButton(app_type),
                    )).with_children(|p| {
                        p.spawn(TextBundle::from_section(format!("{:?}", app_type), TextStyle { font_size: 18.0, ..default() }));
                    });
                }
            });

            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(85.0),
                    height: Val::Percent(100.0),
                    margin: UiRect::all(Val::Px(10.0)),
                    flex_direction: FlexDirection::Column, 
                    ..default()
                },
                ..default()
            }).with_children(|parent| {
                parent.spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            display: Display::None, 
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        ..default()
                    },
                    DashboardPanel,
                )).with_children(|dashboard_content| {
                    dashboard_content.spawn((
                        NodeBundle {
                            style: Style {
                                flex_grow: 0.3, 
                                padding: UiRect::all(Val::Px(5.0)),
                                ..default()
                            },
                            background_color: Color::rgba(0.0, 0.0, 0.0, 0.3).into(),
                            ..default()
                        },
                        NotificationsPanel,
                    )).with_children(|notifications_parent| {
                         notifications_parent.spawn(TextBundle::from_section("Notifications Appear Here", TextStyle {font_size: 16.0, color: Color::GRAY, ..default()}));
                    });

                    dashboard_content.spawn((
                        NodeBundle {
                            style: Style {
                                flex_grow: 0.7, 
                                ..default()
                            },
                            ..default()
                        },
                        AnalyticsGraphPanel,
                    )).with_children(|graph_parent| {
                        graph_parent.spawn((
                            NodeBundle {
                                style: Style {
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                background_color: Color::rgba(0.1, 0.1, 0.1, 0.5).into(), 
                                ..default()
                            },
                            GraphArea, 
                        ));
                    });
                });

                parent.spawn((
                    NodeBundle {
                        style: Style { 
                            width: Val::Percent(100.0), 
                            height: Val::Percent(100.0), 
                            display: Display::None, 
                            flex_direction: FlexDirection::Column, 
                            ..default()
                        },
                        ..default()
                    },
                    ConstructionPanel,
                )).with_children(|construction_panel_content| {
                    construction_panel_content.spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Px(40.0), 
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ..default()
                    }).with_children(|tabs_bar| {
                        let categories = [ConstructionCategory::Operations, ConstructionCategory::Habitation, ConstructionCategory::Legacy];
                        for category in categories {
                            tabs_bar.spawn((
                                ButtonBundle {
                                    style: Style {
                                        padding: UiRect::all(Val::Px(8.0)),
                                        margin: UiRect::horizontal(Val::Px(5.0)),
                                        ..default()
                                    },
                                    background_color: NORMAL_BUTTON.into(),
                                    ..default()
                                },
                                ConstructionCategoryTab(category),
                            )).with_children(|button_content| {
                                button_content.spawn(TextBundle::from_section(format!("{:?}", category), TextStyle {font_size: 16.0, ..default()}));
                            });
                        }
                    });

                    construction_panel_content.spawn((
                        NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                flex_grow: 1.0, 
                                flex_direction: FlexDirection::Column, 
                                padding: UiRect::all(Val::Px(5.0)),
                                ..default()
                            },
                            background_color: Color::rgba(0.0, 0.0, 0.0, 0.2).into(),
                            ..default()
                        },
                        ConstructionItemListPanel,
                    )).with_children(|list_panel| {
                        list_panel.spawn(TextBundle::from_section("Select a category to see buildable items.", TextStyle {font_size: 18.0, color: Color::GRAY, ..default()}));
                    });

                    construction_panel_content.spawn((
                        NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                height: Val::Px(100.0), 
                                padding: UiRect::all(Val::Px(5.0)),
                                ..default()
                            },
                            background_color: Color::rgba(0.0, 0.0, 0.0, 0.4).into(),
                            ..default()
                        },
                        ConstructionItemDetailsPanel,
                    )).with_children(|details_panel| {
                        details_panel.spawn(TextBundle::from_section("Select an item to see details.", TextStyle {font_size: 16.0, color: Color::GRAY, ..default()}));
                    });
                });
                
                parent.spawn((
                    NodeBundle {
                        style: Style { width: Val::Percent(100.0), height: Val::Percent(100.0), display: Display::None, flex_direction: FlexDirection::Column, padding: UiRect::all(Val::Px(10.0)), ..default()}, // Added flex_direction and padding
                        ..default()
                    },
                    ColonyStatusPanel,
                )).with_children(|p| { 
                    p.spawn(TextBundle::from_section("Colony Status Overview", TextStyle {font_size: 24.0, ..default()}).with_style(Style{margin: UiRect::bottom(Val::Px(15.0)), ..default()})); 
                    // Add AdminSpireStatusText here
                    p.spawn((
                        TextBundle::from_section("Admin Spire: Not Constructed", TextStyle {font_size: 18.0, color: Color::WHITE, ..default()})
                            .with_style(Style{ margin: UiRect::bottom(Val::Px(10.0)), ..default()}),
                        AdminSpireStatusText,
                    ));
                    // Placeholder for other colony status elements
                    p.spawn(TextBundle::from_section("More status details will appear here...", TextStyle {font_size: 16.0, color: Color::GRAY, ..default()}));
                });

                parent.spawn((
                    NodeBundle {
                        style: Style { width: Val::Percent(100.0), height: Val::Percent(100.0), display: Display::None, justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default()},
                        ..default()
                    },
                    ResearchPanel,
                )).with_children(|p| { p.spawn(TextBundle::from_section("Research App Content", TextStyle {font_size: 24.0, ..default()})); });
            });
        });
    });
}

fn draw_graph_gizmos(
    mut gizmos: Gizmos,
    graph_data: Res<GraphData>,
    graph_area_query: Query<(&Node, &GlobalTransform), With<GraphArea>>,
) {
    if graph_data.history.is_empty() { return; }

    let (graph_node, transform) = graph_area_query.single();
    let graph_area = graph_node.size();
    
    let bottom_left = transform.translation().truncate() - graph_area / 2.0;

    let max_val = graph_data.history.iter().fold(100.0f32, |max_so_far, stats| {
        max_so_far
            .max(stats.total_housing as f32)
            .max(stats.total_jobs as f32)
            .max(stats.health_capacity as f32)
            .max(stats.police_capacity as f32)
            .max(stats.credits as f32)      
            .max(stats.net_power.abs())     
            .max(stats.happiness as f32)    
            .max(stats.nutrient_paste as f32) 
    });

    let graph_lines: [(Color, Box<dyn GraphableFn>); 8] = [
        (HOUSING_COLOR, Box::new(|stats: &ColonyStats| stats.total_housing as f32)),
        (JOBS_COLOR, Box::new(|stats: &ColonyStats| stats.total_jobs as f32)),
        (HEALTH_COLOR, Box::new(|stats: &ColonyStats| stats.health_capacity as f32)),
        (POLICE_COLOR, Box::new(|stats: &ColonyStats| stats.police_capacity as f32)),
        (CREDITS_COLOR, Box::new(|stats: &ColonyStats| stats.credits as f32)),
        (NET_POWER_COLOR, Box::new(|stats: &ColonyStats| stats.net_power as f32)),
        (HAPPINESS_COLOR, Box::new(|stats: &ColonyStats| stats.happiness as f32)),
        (NUTRIENT_PASTE_COLOR, Box::new(|stats: &ColonyStats| stats.nutrient_paste as f32)),
    ];

    for (color, get_value) in graph_lines.iter() { 
        let mut points: Vec<Vec2> = Vec::new();
        for (i, stats) in graph_data.history.iter().enumerate() {
            let value = get_value(stats); 
            let point_x_offset = (i as f32 / graph_data.history.len().max(1) as f32) * graph_area.x;
            let x = graph_area.x - point_x_offset;
            let y_scaled = if max_val == 0.0 { 0.0 } else { (value / max_val) * graph_area.y };
            if x >= 0.0 && x <= graph_area.x {
                 points.push(bottom_left + Vec2::new(x, y_scaled.clamp(0.0, graph_area.y)));
            }
        }
        if points.len() > 1 {
            gizmos.linestrip_2d(points, *color); 
        }
    }
}

fn update_admin_spire_status_system(
    game_state: Res<GameState>,
    mut query: Query<&mut Text, With<AdminSpireStatusText>>,
) {
    if let Ok(mut text) = query.get_single_mut() {
        if let Some(spire) = &game_state.administrative_spire {
            let current_tier_info = &spire.available_tiers[spire.current_tier_index];
            let has_paste = game_state.current_resources.get(&ResourceType::NutrientPaste).unwrap_or(&0.0) > &0.0;

            let next_tier_requires_paste = if spire.current_tier_index < spire.available_tiers.len() - 1 {
                spire.available_tiers[spire.current_tier_index + 1].nutrient_paste_link_required
            } else {
                false // At max tier, no "next" requirement
            };
            
            let status_message = if spire.current_tier_index < spire.available_tiers.len() - 1 {
                 format!(
                    "Admin Spire: {} (Tier {})\nNutrient Paste: {} (Required for next upgrade: {})",
                    current_tier_info.name,
                    spire.current_tier_index + 1, // Display as 1-indexed tier
                    if has_paste { "Available" } else { "Not Available" },
                    if next_tier_requires_paste { "Yes" } else { "No" }
                )
            } else {
                format!(
                    "Admin Spire: {} (Tier {})\nNutrient Paste: {}\n(Max Tier Reached)",
                    current_tier_info.name,
                    spire.current_tier_index + 1,
                    if has_paste { "Available" } else { "Not Available" }
                )
            };
            text.sections[0].value = status_message;

        } else {
            text.sections[0].value = "Admin Spire: Not Constructed".to_string();
        }
    }
}


fn zone_button_system(
    mut interaction_query: Query<(&Interaction, &BuildZoneButton, &mut BackgroundColor), Changed<Interaction>>,
    mut game_state: ResMut<GameState>,
    mut log: ResMut<MessageLog>,
) {
    for (interaction, button_data, mut color) in &mut interaction_query {
        let zone_type = button_data.zone_type;
        let tier_index = button_data.tier_index;

        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                if !game_state.unlocked_techs.contains(&Tech::BasicConstructionProtocols) {
                    log.message = "Requires Basic Construction Protocols.".to_string();
                    continue; 
                }
                let initial_credits = game_state.credits;
                let initial_zone_count = game_state.zones.len();
                game_state::add_zone(&mut game_state, zone_type, tier_index);
                if game_state.zones.len() > initial_zone_count && game_state.credits < initial_credits {
                    let all_tiers_for_type = game_state::get_zone_tiers(zone_type);
                    let name = all_tiers_for_type.get(tier_index).map_or_else(
                        || format!("{:?} Tier {}", zone_type, tier_index),
                        |t| t.name.clone()
                    );
                    log.message = format!("{} developed.", name);
                } else if game_state.credits == initial_credits && game_state.zones.len() == initial_zone_count {
                    let all_tiers_for_type = game_state::get_zone_tiers(zone_type);
                    if let Some(tier_info) = all_tiers_for_type.get(tier_index) {
                        log.message = format!("Failed: Need {} credits for {}.", tier_info.construction_credits_cost, tier_info.name);
                    } else {
                        log.message = format!("Failed to develop {:?} (Tier {}). Check credits/console.", zone_type, tier_index);
                    }
                } else {
                    log.message = format!("Action for {:?} (Tier {}). Check console.", zone_type, tier_index);
                }
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                let tiers = game_state::get_zone_tiers(button_data.zone_type);
                if let Some(tier_info) = tiers.get(button_data.tier_index) {
                    let mut message = format!("{}: Cost {} Credits. Upkeep: {} Credits.", tier_info.name, tier_info.construction_credits_cost, tier_info.upkeep_cost);
                    if !game_state.unlocked_techs.contains(&Tech::BasicConstructionProtocols) {
                        message.push_str(" (Req: Basic Construction Protocols)");
                    }
                    log.message = message;
                } else {
                    log.message = format!("Hovering Zone {:?} Tier {}.", button_data.zone_type, button_data.tier_index);
                }
            }
            Interaction::None => { *color = NORMAL_BUTTON.into(); }
        }
    }
}

fn service_building_button_system(
    mut interaction_query: Query<(&Interaction, &BuildServiceBuildingButton, &mut BackgroundColor), Changed<Interaction>>,
    mut game_state: ResMut<GameState>,
    mut log: ResMut<MessageLog>,
) {
    for (interaction, button_data, mut color) in &mut interaction_query {
        let service_type = button_data.service_type;
        let tier_index = button_data.tier_index;

        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                if !game_state.unlocked_techs.contains(&Tech::BasicConstructionProtocols) {
                    log.message = "Requires Basic Construction Protocols.".to_string();
                    continue; 
                }
                let initial_credits = game_state.credits;
                let initial_building_count = game_state.service_buildings.len();
                game_state::add_service_building(&mut game_state, service_type, tier_index, None);
                if game_state.service_buildings.len() > initial_building_count && game_state.credits < initial_credits {
                    let tiers_for_type = game_state::get_service_building_tiers(service_type);
                    let name = tiers_for_type.get(tier_index).map_or_else(
                        || format!("{:?} Tier {}", service_type, tier_index),
                        |t| t.name.clone()
                    );
                    log.message = format!("{} constructed.", name);
                } else if game_state.credits == initial_credits && game_state.service_buildings.len() == initial_building_count {
                    let all_tiers_for_type = game_state::get_service_building_tiers(service_type);
                    if let Some(tier_info) = all_tiers_for_type.get(tier_index) {
                        log.message = format!("Failed: Need {} credits for {}.", tier_info.construction_credits_cost, tier_info.name);
                    } else {
                        log.message = format!("Failed to build {:?} (Tier {}). Check credits/console.", service_type, tier_index);
                    }
                } else {
                    log.message = format!("Action for {:?} (Tier {}). Check console.", service_type, tier_index);
                }
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                let tiers = game_state::get_service_building_tiers(button_data.service_type);
                if let Some(tier_info) = tiers.get(button_data.tier_index) {
                    let mut message = format!("{}: Cost {} Credits. Upkeep: {} Credits.", tier_info.name, tier_info.construction_credits_cost, tier_info.upkeep_cost);
                    if !game_state.unlocked_techs.contains(&Tech::BasicConstructionProtocols) {
                        message.push_str(" (Req: Basic Construction Protocols)");
                    }
                    log.message = message;
                } else {
                    log.message = format!("Hovering Service {:?} Tier {}.", button_data.service_type, button_data.tier_index);
                }
            }
            Interaction::None => { *color = NORMAL_BUTTON.into(); }
        }
    }
}

fn button_interaction_system(
    mut interaction_query: Query<(&Interaction, &BuildButton, &mut BackgroundColor), Changed<Interaction>>,
    mut game_state: ResMut<GameState>,
    mut commands: Commands,
    mut log: ResMut<MessageLog>,
) {
    for (interaction, build_button, mut color) in &mut interaction_query {
        let building_type = build_button.0;
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                if (building_type == BuildingType::BasicDwelling || building_type == BuildingType::WellnessPost || building_type == BuildingType::SecurityStation) 
                   && !game_state.unlocked_techs.contains(&Tech::BasicConstructionProtocols) {
                    log.message = "Requires Basic Construction Protocols".to_string();
                    continue; 
                }
                
                let costs = game_state.building_costs.get(&building_type).unwrap().clone();
                if costs.iter().all(|(res, &cost)| game_state.current_resources.get(res).unwrap_or(&0.0) >= &cost) {
                    for (res, cost) in &costs { *game_state.current_resources.get_mut(res).unwrap() -= cost; }
                    log.message = format!("Construction started: {:?}", building_type);
                    match building_type {
                        BuildingType::BioDome => { commands.spawn(BioDome { power_consumption: 10, production_rate: 5.0 }); }
                        BuildingType::Extractor => { commands.spawn(Extractor { power_consumption: 15, resource_type: ResourceType::FerrocreteOre, extraction_rate: 2.5 }); }
                        BuildingType::PowerRelay => { commands.spawn(PowerRelay { power_output: 50 }); }
                        BuildingType::StorageSilo => { commands.spawn(StorageSilo { capacity: 1000 }); }
                        BuildingType::ResearchInstitute => { commands.spawn(ResearchInstitute { power_consumption: 5 }); } 
                        BuildingType::BasicDwelling => { commands.spawn(BasicDwelling { housing_capacity: 100 }); }
                        BuildingType::WellnessPost => { commands.spawn(WellnessPost { health_capacity: 50, jobs_provided: 5 }); }
                        BuildingType::SecurityStation => { commands.spawn(SecurityStation { police_capacity: 50, jobs_provided: 5 }); }
                        BuildingType::Fabricator => {
                            game_state::add_fabricator(&mut game_state, 0); 
                            log.message = "Fabricator construction initiated.".to_string();
                        }
                        BuildingType::ProcessingPlant => {
                            game_state::add_processing_plant(&mut game_state, 0);
                            log.message = "Processing Plant construction initiated.".to_string();
                        }
                    }
                } else {
                    log.message = "Not enough material resources!".to_string();
                }
            }
            Interaction::Hovered => { *color = HOVERED_BUTTON.into(); }
            Interaction::None => { *color = NORMAL_BUTTON.into(); }
        }
    }
}

fn research_button_system(
    mut interaction_query: Query<(&Interaction, &ResearchButton, &mut BackgroundColor), Changed<Interaction>>,
    mut game_state: ResMut<GameState>,
    mut log: ResMut<MessageLog>,
    institute_q: Query<&ResearchInstitute>,
) {
    for (interaction, research_button, mut color) in &mut interaction_query {
        let tech = research_button.0;
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                if institute_q.is_empty() {
                    log.message = "Must build a Research Institute first.".to_string();
                } else if game_state.research_progress.is_some() {
                    log.message = "Research already in progress.".to_string();
                } else if game_state.unlocked_techs.contains(&tech) {
                    log.message = "Technology already researched.".to_string();
                } else {
                    let credit_cost = *game_state.tech_costs.get(&tech).unwrap_or(&0_u32); 
                    if game_state.credits >= credit_cost as f64 {
                        game_state.credits -= credit_cost as f64;
                        log.message = format!("Researching {:?} for {} Credits...", tech, credit_cost);
                        game_state.research_progress = Some((tech, 0.0));
                    } else {
                        log.message = format!("Not enough Credits to research {:?}. Cost: {} Credits, Available: {:.0}", tech, credit_cost, game_state.credits);
                    }
                }
            }
            Interaction::Hovered => { *color = HOVERED_BUTTON.into(); }
            Interaction::None => { *color = NORMAL_BUTTON.into(); }
        }
    }
}

fn update_text_display(
    game_state: Res<GameState>,
    stats: Res<ColonyStats>,
    log: Res<MessageLog>,
    mut text_queries: ParamSet<(
        Query<&mut Text, With<PowerText>>,           
        Query<(&mut Text, &ResourceText)>,           
        Query<(&mut Text, &ColonyStatText)>,         
        Query<&mut Text, With<MessageText>>,         
        Query<&mut Text, With<CreditsText>>,         
        Query<&mut Text, With<PopulationText>>,     
        Query<(&mut Text, &CoreResourceText)>,       
        Query<&mut Text, With<ColonyHappinessText>>, 
    )>,
) {
    for mut text in text_queries.p4().iter_mut() {
        text.sections[0].value = format!("Cr. {:.0}", game_state.credits);
    }

    for mut text in text_queries.p0().iter_mut() {
        let net_power = game_state.total_generated_power - game_state.total_consumed_power;
        let stored_power = *game_state.current_resources.get(&ResourceType::Power).unwrap_or(&0.0);
        let max_power_storage = 5000.0; 
        text.sections[0].value = format!("Power - Net: {:+.0} | Stored: {:.0}/{:.0}", net_power, stored_power, max_power_storage);
        text.sections[0].style.color = if net_power < 0.0 { Color::RED } else { Color::CYAN };
    }

    for mut text in text_queries.p5().iter_mut() {
        text.sections[0].value = format!("Inhabitants: {} / {}", game_state.total_inhabitants, game_state.available_housing_capacity);
        let housing_ratio = if game_state.available_housing_capacity > 0 {
            game_state.total_inhabitants as f32 / game_state.available_housing_capacity as f32
        } else if game_state.total_inhabitants > 0 { 
            2.0 
        } else { 
            0.0 
        };
        text.sections[0].style.color = if housing_ratio >= 0.9 { Color::rgb(1.0, 0.9, 0.3) } else { Color::WHITE }; 
    }

    for (mut text, core_resource_marker) in text_queries.p6().iter_mut() {
        let amount = game_state.current_resources.get(&core_resource_marker.0).unwrap_or(&0.0);
        text.sections[0].value = format!("{:?}: {:.0}", core_resource_marker.0, amount);
    }

    for mut text in text_queries.p7().iter_mut() {
        let happiness_icon = match game_state.colony_happiness {
            h if h >= 85.0 => "😊",
            h if h >= 50.0 => "😐",
            _ => "☹️",
        };
        text.sections[0].value = format!("{} {:.0}%", happiness_icon, game_state.colony_happiness);
        text.sections[0].style.color = match game_state.colony_happiness {
            h if h >= 85.0 => Color::rgb(0.3, 1.0, 0.3), 
            h if h >= 50.0 => Color::rgb(1.0, 0.9, 0.3), 
            _ => Color::rgb(1.0, 0.3, 0.3),              
        };
    }
    
    for (mut text, resource_marker) in text_queries.p1().iter_mut() {
        let amount = game_state.current_resources.get(&resource_marker.0).unwrap_or(&0.0);
        text.sections[0].value = format!("{:?}: {:.0}", resource_marker.0, amount); 
    }
    
    for (mut text, stat_marker) in text_queries.p2().iter_mut() {
        text.sections[0].value = match stat_marker.0 {
            StatType::Housing => format!("Housing: {}", stats.total_housing), 
            StatType::Jobs => format!("Jobs: {}", stats.total_jobs),
            StatType::Health => format!("Health: {}", stats.health_capacity),
            StatType::Police => format!("Police: {}", stats.police_capacity),
        };
    }
    
    if log.is_changed() {
        if let Ok(mut text) = text_queries.p3().get_single_mut() { 
            text.sections[0].value = log.message.clone();
        }
    }
}

fn manage_app_panels_visibility(
    current_app: Res<CurrentApp>,
    mut panel_queries: ParamSet<(
        Query<&mut Style, With<DashboardPanel>>,
        Query<&mut Style, With<ConstructionPanel>>,
        Query<&mut Style, With<ColonyStatusPanel>>,
        Query<&mut Style, With<ResearchPanel>>,
    )>,
) {
    for mut style in panel_queries.p0().iter_mut() { style.display = Display::None; }
    for mut style in panel_queries.p1().iter_mut() { style.display = Display::None; }
    for mut style in panel_queries.p2().iter_mut() { style.display = Display::None; }
    for mut style in panel_queries.p3().iter_mut() { style.display = Display::None; }

    match current_app.0 {
        AppType::Dashboard => {
            if let Ok(mut style) = panel_queries.p0().get_single_mut() {
                style.display = Display::Flex;
            }
        }
        AppType::Construction => {
            if let Ok(mut style) = panel_queries.p1().get_single_mut() {
                style.display = Display::Flex;
            }
        }
        AppType::ColonyStatus => {
            if let Ok(mut style) = panel_queries.p2().get_single_mut() {
                style.display = Display::Flex;
            }
        }
        AppType::Research => {
            if let Ok(mut style) = panel_queries.p3().get_single_mut() {
                style.display = Display::Flex;
            }
        }
    }
}

fn update_dashboard_notifications_system(
    current_app: Res<CurrentApp>,
    game_state: Res<GameState>,
    notifications_panel_query: Query<Entity, With<NotificationsPanel>>,
    mut commands: Commands,
) {
    if current_app.0 != AppType::Dashboard {
        return;
    }

    if game_state.is_changed() || current_app.is_changed() {
        if let Ok(panel_entity) = notifications_panel_query.get_single() {
            commands.entity(panel_entity).despawn_descendants();
            for event in game_state.notifications.iter().take(10) { 
                commands.entity(panel_entity).with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        format!("[{:.1}] {}", event.timestamp, event.message), 
                        TextStyle { font_size: 16.0, color: Color::WHITE, ..default() }
                    ).with_style(Style{ margin: UiRect::bottom(Val::Px(2.0)), ..default()}));
                });
            }
        }
    }
}

const ACTIVE_BUTTON_COLOR: Color = Color::rgb(0.2, 0.5, 0.2); 
const ACTIVE_CONSTRUCTION_TAB_COLOR: Color = Color::rgb(0.2, 0.5, 0.2); 

fn construction_category_tab_system(
    mut current_category: ResMut<CurrentConstructionCategory>,
    mut button_query: Query<(&Interaction, &ConstructionCategoryTab, &mut BackgroundColor), With<Button>>,
) {
    for (interaction, tab, mut bg_color) in button_query.iter_mut() {
        if tab.0 == current_category.0 {
            *bg_color = ACTIVE_CONSTRUCTION_TAB_COLOR.into();
        } else {
            *bg_color = NORMAL_BUTTON.into();
        }

        match *interaction {
            Interaction::Pressed => {
                *bg_color = PRESSED_BUTTON.into();
                if current_category.0 != tab.0 {
                    current_category.0 = tab.0;
                    println!("Switched to construction category: {:?}", current_category.0);
                }
            }
            Interaction::Hovered => {
                if tab.0 != current_category.0 {
                    *bg_color = HOVERED_BUTTON.into();
                }
            }
            Interaction::None => {}
        }
    }
}

fn update_construction_list_system(
    current_app: Res<CurrentApp>,
    current_category: Res<CurrentConstructionCategory>,
    mut item_list_panel_query: Query<Entity, With<ConstructionItemListPanel>>,
    game_state: Res<GameState>, 
    mut commands: Commands,
) {
    if current_app.0 != AppType::Construction {
        return;
    }

    if current_category.is_changed() || (current_app.is_changed() && current_app.0 == AppType::Construction) {
        if let Ok(panel_entity) = item_list_panel_query.get_single_mut() {
            commands.entity(panel_entity).despawn_descendants(); 
            let building_meta_map = get_building_metadata();
            let mut found_items = false;
            commands.entity(panel_entity).with_children(|parent| {
                for building_type_ref in ALL_BUILDING_TYPES.iter() {
                    let building_type = *building_type_ref; 
                    if let Some(meta) = building_meta_map.get(&building_type) {
                        if meta.category == current_category.0 {
                            found_items = true;
                            let tech_ok = meta.required_tech.map_or(true, |tech| game_state.unlocked_techs.contains(&tech));
                            if tech_ok { 
                                let (can_afford_credits, can_afford_materials) = check_affordability(&game_state, building_type);
                                let button_color = if can_afford_credits && can_afford_materials { NORMAL_BUTTON } else { Color::rgba(0.5, 0.15, 0.15, 0.8) };
                                parent.spawn((
                                    ButtonBundle {
                                        style: Style { 
                                            width: Val::Percent(90.0), 
                                            padding: UiRect::all(Val::Px(5.0)), 
                                            margin: UiRect::all(Val::Px(2.0)), 
                                            justify_content: JustifyContent::Center, 
                                            align_items: AlignItems::Center, 
                                            ..default() 
                                        },
                                        background_color: button_color.into(),
                                        ..default()
                                    },
                                    ConstructionItemButton(building_type) 
                                )).with_children(|p| {
                                    p.spawn(TextBundle::from_section(meta.name, TextStyle { font_size: 16.0, ..default() }));
                                });
                            }
                        }
                    }
                }
                if !found_items {
                    parent.spawn(TextBundle::from_section(
                        format!("No items currently available in {:?} category.", current_category.0), 
                        TextStyle { font_size: 16.0, color: Color::GRAY, ..default() }
                    ));
                }
            });
        }
    }
}

fn check_affordability(game_state: &Res<GameState>, building_type: GameBuildingType) -> (bool, bool) {
    let can_afford_credits = true; 
    let mut can_afford_materials = true;

    match building_type {
        GameBuildingType::Fabricator | GameBuildingType::ProcessingPlant | GameBuildingType::BasicDwelling => {
            if let Some(material_costs) = game_state.building_costs.get(&building_type) {
                for (res, &required_amount) in material_costs {
                    if game_state.current_resources.get(res).unwrap_or(&0.0) < &required_amount {
                        can_afford_materials = false;
                        break;
                    }
                }
            }
        }
        _ => { 
            if let Some(material_costs) = game_state.building_costs.get(&building_type) {
                for (res, &required_amount) in material_costs {
                    if game_state.current_resources.get(res).unwrap_or(&0.0) < &required_amount {
                        can_afford_materials = false;
                        break;
                    }
                }
            }
        }
    }
    (can_afford_credits, can_afford_materials)
}

fn construction_item_interaction_system(
    mut selected_building_res: ResMut<SelectedBuilding>,
    game_state: Res<GameState>, 
    mut button_query: Query<(&Interaction, &ConstructionItemButton, &mut BackgroundColor), With<Button>>,
) {
    for (interaction, item_button, mut bg_color) in button_query.iter_mut() {
        let building_type = item_button.0;
        let is_selected = selected_building_res.0 == Some(building_type);

        if is_selected {
            *bg_color = Color::rgb(0.2, 0.6, 0.2).into(); 
        } else {
            let (can_afford_credits, can_afford_materials) = check_affordability(&game_state, building_type);
            if can_afford_credits && can_afford_materials {
                *bg_color = NORMAL_BUTTON.into();
            } else {
                *bg_color = Color::rgba(0.5, 0.15, 0.15, 0.8).into(); 
            }
        }

        match *interaction {
            Interaction::Pressed => {
                *bg_color = PRESSED_BUTTON.into(); 
                if is_selected {
                    selected_building_res.0 = None; 
                } else {
                    selected_building_res.0 = Some(building_type);
                }
            }
            Interaction::Hovered => {
                if !is_selected {
                    *bg_color = HOVERED_BUTTON.into();
                }
            }
            Interaction::None => {}
        }
    }
}

fn update_construction_details_panel_system(
    selected_building: Res<SelectedBuilding>,
    game_state: Res<GameState>,
    mut details_panel_query: Query<Entity, With<ConstructionItemDetailsPanel>>,
    mut commands: Commands,
) {
    if !selected_building.is_changed() && !game_state.is_changed(){ 
        return;
    }

    if let Ok(details_panel_entity) = details_panel_query.get_single_mut() {
        commands.entity(details_panel_entity).despawn_descendants();
        let building_meta_map = get_building_metadata();

        if let Some(building_type) = selected_building.0 {
            if let Some(meta) = building_meta_map.get(&building_type) {
                commands.entity(details_panel_entity).with_children(|parent| {
                    parent.spawn(TextBundle::from_section(meta.name, TextStyle { font_size: 24.0, ..default() }).with_style(Style{ margin: UiRect::bottom(Val::Px(10.0)), ..default()}));
                    parent.spawn(TextBundle::from_section("A standard construction offering from Nexus Core.", TextStyle { font_size: 16.0, ..default() }).with_style(Style{ margin: UiRect::bottom(Val::Px(8.0)), ..default()}));
                    parent.spawn(TextBundle::from_section("Costs:", TextStyle { font_size: 18.0, ..default() }).with_style(Style{ margin: UiRect::bottom(Val::Px(3.0)), ..default()}));
                    parent.spawn(TextBundle::from_section(" - Credits: Varies (see build action)", TextStyle { font_size: 16.0, ..default() }).with_style(Style{ margin: UiRect { left: Val::Px(10.0), bottom: Val::Px(2.0), right: Val::Px(0.0), top: Val::Px(0.0) }, ..default()}));
                    if let Some(material_costs) = game_state.building_costs.get(&building_type) {
                        if material_costs.is_empty() { 
                            parent.spawn(TextBundle::from_section(" - Materials: None", TextStyle { font_size: 16.0, ..default() }).with_style(Style{ margin: UiRect { left: Val::Px(10.0), bottom: Val::Px(2.0), right: Val::Px(0.0), top: Val::Px(0.0) }, ..default()}));
                        }
                        for (res, amount) in material_costs { 
                            parent.spawn(TextBundle::from_section(format!(" - {:?}: {}", res, amount), TextStyle { font_size: 16.0, ..default() }).with_style(Style{ margin: UiRect { left: Val::Px(10.0), bottom: Val::Px(2.0), right: Val::Px(0.0), top: Val::Px(0.0) }, ..default()})); 
                        }
                    } else { 
                        parent.spawn(TextBundle::from_section(" - Materials: None specified", TextStyle { font_size: 16.0, ..default() }).with_style(Style{ margin: UiRect { left: Val::Px(10.0), bottom: Val::Px(2.0), right: Val::Px(0.0), top: Val::Px(0.0) }, ..default()}));
                    }
                    parent.spawn(TextBundle::from_section("Requirements:", TextStyle { font_size: 18.0, ..default() }).with_style(Style{ margin: UiRect { top: Val::Px(8.0), bottom: Val::Px(3.0), left: Val::Px(0.0), right: Val::Px(0.0) }, ..default()}));
                    if let Some(tech) = meta.required_tech { 
                        parent.spawn(TextBundle::from_section(format!(" - Tech: {:?}", tech), TextStyle { font_size: 16.0, ..default() }).with_style(Style{ margin: UiRect { left: Val::Px(10.0), bottom: Val::Px(2.0), right: Val::Px(0.0), top: Val::Px(0.0) }, ..default()}));
                    } else { 
                        parent.spawn(TextBundle::from_section(" - Tech: None", TextStyle { font_size: 16.0, ..default() }).with_style(Style{ margin: UiRect { left: Val::Px(10.0), bottom: Val::Px(2.0), right: Val::Px(0.0), top: Val::Px(0.0) }, ..default()}));
                    }
                    if let Some(dp) = meta.required_dp { 
                        parent.spawn(TextBundle::from_section(format!(" - Min. DP: {:?}", dp), TextStyle { font_size: 16.0, ..default() }).with_style(Style{ margin: UiRect { left: Val::Px(10.0), bottom: Val::Px(2.0), right: Val::Px(0.0), top: Val::Px(0.0) }, ..default()}));
                    }
                    parent.spawn((
                        ButtonBundle {
                            style: Style { width: Val::Percent(50.0), padding: UiRect::all(Val::Px(8.0)), margin: UiRect::top(Val::Px(15.0)), justify_content: JustifyContent::Center, align_self: AlignSelf::Center, ..default() },
                            background_color: NORMAL_BUTTON.into(),
                            ..default()
                        },
                        ConfirmBuildButton(building_type)
                    )).with_children(|p| { p.spawn(TextBundle::from_section("Initiate Construction", TextStyle { font_size: 18.0, ..default() })); });
                });
            }
        } else {
            commands.entity(details_panel_entity).with_children(|parent| {
                parent.spawn(TextBundle::from_section("Select an item from the list to see details.", TextStyle { font_size: 16.0, color: Color::GRAY, ..default() }));
            });
        }
    }
}

fn construction_interaction_system(
    mut interaction_query: Query<(&Interaction, &ConfirmBuildButton), (Changed<Interaction>, With<Button>)>,
    mut game_state: ResMut<GameState>,
    mut commands: Commands,
    mut selected_building: ResMut<SelectedBuilding>, 
    time: Res<Time>,
) {
    for (interaction, confirm_button) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            let building_type = confirm_button.0;
            let building_meta_map = get_building_metadata();
            let meta = if let Some(m) = building_meta_map.get(&building_type) { m } else { 
                game_state::add_notification(&mut game_state, format!("Error: No metadata for {:?}", building_type), time.elapsed_seconds_f64());
                return; 
            };

            if let Some(req_tech) = meta.required_tech {
                if !game_state.unlocked_techs.contains(&req_tech) {
                    game_state::add_notification(&mut game_state, format!("Failed: {:?} requires tech {:?}", meta.name, req_tech), time.elapsed_seconds_f64());
                    return;
                }
            }
            
            let mut can_afford_materials = true;
            if let Some(material_costs) = game_state.building_costs.get(&building_type) {
                for (res, &required_amount) in material_costs {
                    if game_state.current_resources.get(res).unwrap_or(&0.0) < &required_amount {
                        can_afford_materials = false;
                        break;
                    }
                }
            }
            if !can_afford_materials {
                game_state::add_notification(&mut game_state, format!("Failed: Not enough materials for {:?}", meta.name), time.elapsed_seconds_f64());
                return;
            }

            let costs_to_deduct: Option<Vec<(ResourceType, f32)>> = 
                game_state.building_costs.get(&building_type)
                .map(|costs_map| {
                    costs_map.iter().map(|(res_type, val)| (*res_type, *val)).collect()
                });

            if let Some(actual_costs) = costs_to_deduct {
                for (res, cost_val) in actual_costs {
                    if let Some(current_res_val) = game_state.current_resources.get_mut(&res) {
                        *current_res_val -= cost_val;
                    }
                }
            }
            
            let mut construction_triggered = false;
            match building_type {
                GameBuildingType::Fabricator => { game_state::add_fabricator(&mut game_state, 0); construction_triggered = true; }
                GameBuildingType::ProcessingPlant => { game_state::add_processing_plant(&mut game_state, 0); construction_triggered = true; }
                GameBuildingType::BasicDwelling => { game_state::add_habitation_structure(&mut game_state, 0); construction_triggered = true; }
                GameBuildingType::Extractor => { commands.spawn(Extractor { power_consumption: 15, resource_type: ResourceType::FerrocreteOre, extraction_rate: 2.5 }); construction_triggered = true; }
                GameBuildingType::BioDome => { commands.spawn(BioDome { power_consumption: 10, production_rate: 5.0 }); construction_triggered = true; }
                GameBuildingType::PowerRelay => { commands.spawn(PowerRelay { power_output: 50 }); construction_triggered = true; }
                GameBuildingType::StorageSilo => { commands.spawn(StorageSilo { capacity: 1000 }); construction_triggered = true; }
                GameBuildingType::ResearchInstitute => { commands.spawn(ResearchInstitute { power_consumption: 5 }); construction_triggered = true; }
                GameBuildingType::WellnessPost => { commands.spawn(WellnessPost { health_capacity: 50, jobs_provided: 5 }); construction_triggered = true; }
                GameBuildingType::SecurityStation => { commands.spawn(SecurityStation { police_capacity: 50, jobs_provided: 5 }); construction_triggered = true; }
                _ => { game_state::add_notification(&mut game_state, format!("Construction for {:?} not yet implemented.", meta.name), time.elapsed_seconds_f64()); }
            }

            if construction_triggered {
                game_state::add_notification(&mut game_state, format!("Construction started: {:?}", meta.name), time.elapsed_seconds_f64());
                selected_building.0 = None; 
            }
        }
    }
}

fn app_drawer_button_system(
    mut current_app_res: ResMut<CurrentApp>,
    mut button_query: Query<(&Interaction, &AppDrawerButton, &mut BackgroundColor), With<Button>>,
) {
    for (interaction, app_button, mut bg_color) in button_query.iter_mut() {
        if app_button.0 == current_app_res.0 {
            *bg_color = ACTIVE_BUTTON_COLOR.into();
        } else {
            *bg_color = NORMAL_BUTTON.into();
        }

        match *interaction {
            Interaction::Pressed => {
                *bg_color = PRESSED_BUTTON.into();
                if current_app_res.0 != app_button.0 {
                    current_app_res.0 = app_button.0;
                    println!("Switched to app: {:?}", current_app_res.0);
                }
            }
            Interaction::Hovered => {
                if app_button.0 != current_app_res.0 {
                    *bg_color = HOVERED_BUTTON.into();
                }
            }
            Interaction::None => {}
        }
    }
}

fn admin_spire_button_system(
    mut button_queries: ParamSet<(
        Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<ConstructSpireButton>)>,
        Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<UpgradeSpireButton>)>,
    )>,
    mut game_state: ResMut<GameState>,
    mut log: ResMut<MessageLog>, // Keep MessageLog for now, or transition to direct notifications
) {
    // Handle Construct Spire Button
    for (interaction, mut color) in button_queries.p0().iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                let credits_before = game_state.credits;
                game_state::construct_administrative_spire(&mut game_state);
                if game_state.administrative_spire.is_some() && game_state.credits < credits_before {
                    log.message = "Administrative Spire constructed.".to_string();
                } else if game_state.administrative_spire.is_some() {
                    log.message = "Administrative Spire already exists.".to_string();
                } else {
                    log.message = "Failed to construct Spire. Check credits/console.".to_string();
                }
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                if game_state.administrative_spire.is_none() {
                    log.message = "Command Post: Cost 1000 Credits.".to_string();
                } else {
                    log.message = "Administrative Spire already constructed.".to_string();
                }
            }
            Interaction::None => { *color = NORMAL_BUTTON.into(); }
        }
    }

    // Handle Upgrade Spire Button
    for (interaction, mut color) in button_queries.p1().iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                if game_state.administrative_spire.is_none() {
                    log.message = "Construct Administrative Spire first.".to_string();
                    continue;
                }
                let current_tier_before_upgrade = game_state.administrative_spire.as_ref().map(|s| s.current_tier_index);
                game_state::upgrade_administrative_spire(&mut game_state); // This function now prints its own failure reasons
                let current_tier_after_upgrade = game_state.administrative_spire.as_ref().map(|s| s.current_tier_index);

                if current_tier_after_upgrade > current_tier_before_upgrade {
                    log.message = "Administrative Spire upgrade successful.".to_string();
                } else if current_tier_after_upgrade == current_tier_before_upgrade {
                    // Specific failure messages are now handled by upgrade_administrative_spire, so a generic one here is okay,
                    // or rely on console output from game_state.rs
                    log.message = "Spire upgrade conditions not met (see console).".to_string();
                     if let Some(spire) = &game_state.administrative_spire {
                        if spire.current_tier_index >= spire.available_tiers.len() -1 {
                            log.message = "Spire already at max tier.".to_string();
                        }
                     }
                } else {
                    log.message = "Error during Spire upgrade (check console).".to_string(); // Should not normally happen
                }
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                if let Some(spire) = &game_state.administrative_spire {
                    if spire.current_tier_index < spire.available_tiers.len() - 1 {
                        let next_tier_info: &AdministrativeSpireTier = &spire.available_tiers[spire.current_tier_index + 1];
                        let mut message = format!("Upgrade to {}: Cost {} Credits.", next_tier_info.name, next_tier_info.upgrade_credits_cost);
                        if next_tier_info.nutrient_paste_link_required {
                            message.push_str(" (Requires Nutrient Paste)");
                        }
                        log.message = message;
                    } else {
                        log.message = "Spire at max tier.".to_string();
                    }
                } else {
                    log.message = "Construct Spire first.".to_string();
                }
            }
            Interaction::None => { *color = NORMAL_BUTTON.into(); }
        }
    }
}