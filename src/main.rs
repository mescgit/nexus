// src/main.rs

use bevy::prelude::*;
use game_state::{
    ColonyStats, GameState, GraphData, ResourceType, Tech, ALL_BUILDING_TYPES,
};
use std::collections::HashMap;

// Import everything needed from game_state
mod game_state;
use game_state::{BuildingType as GameBuildingType, DevelopmentPhase, ServiceType, ZoneType};

// --- Color & Style Constants ---
const PANEL_BACKGROUND: Color = Color::rgba(0.02, 0.02, 0.05, 0.85);
const BORDER_COLOR: Color = Color::rgba(0.2, 0.5, 1.0, 0.9);
const NORMAL_BUTTON: Color = Color::rgba(0.15, 0.15, 0.2, 0.8);
const HOVERED_BUTTON: Color = Color::rgba(0.25, 0.25, 0.3, 1.0);
const PRESSED_BUTTON: Color = Color::rgba(0.35, 0.75, 0.35, 1.0);
const ACTIVE_BUTTON: Color = Color::rgba(0.1, 0.4, 0.8, 1.0);
const PRIMARY_TEXT_COLOR: Color = Color::rgba(0.9, 0.9, 0.9, 0.9);
const LABEL_TEXT_COLOR: Color = Color::rgba(0.7, 0.7, 0.8, 0.9);
const DISABLED_BUTTON: Color = Color::rgba(0.3, 0.1, 0.1, 0.8);

// --- UI Marker Components ---

// App Panels
#[derive(Component)]
struct DashboardPanel;
#[derive(Component)]
struct ConstructionPanel;
#[derive(Component)]
struct ColonyStatusPanel;
#[derive(Component)]
struct ResearchPanel;

// App Drawer
#[derive(Component)]
struct AppDrawerButton(AppType);

// Top Status Ticker
#[derive(Component)]
struct CreditsText;
#[derive(Component)]
struct PowerText;
#[derive(Component)]
struct PopulationText;
#[derive(Component)]
struct WorkforceText;
#[derive(Component)]
struct CoreResourceText(ResourceType);
#[derive(Component)]
struct ColonyHappinessText;

// Dashboard Components
#[derive(Component)]
struct NotificationsPanel;
#[derive(Component)]
struct AnalyticsGraphPanel;
#[derive(Component)]
struct GraphArea;
#[derive(Component)]
struct AdminSpireInfoPanel;
#[derive(Component)]
struct AdminSpireTierText;
#[derive(Component)]
struct ConstructSpireButton;
#[derive(Component)]
struct UpgradeSpireButton;
#[derive(Component)]
struct ManagedStructuresPanel;
#[derive(Component)]
struct ZoneListButton(String); // Holds Zone ID
#[derive(Component)]
struct ZoneDetailsPanel; // Panel for selected zone's details and actions
#[derive(Component)]
struct UpgradeZoneButton(String); // Holds Zone ID for upgrade action
#[derive(Component)]
struct RemoveZoneButton(String); // Holds Zone ID for remove action
#[derive(Component)]
struct AssignSpecialistToZoneButton(String); // Holds Zone ID
#[derive(Component)]
struct UnassignSpecialistFromZoneButton(String); // Holds Zone ID


// Construction Components
#[derive(Component)]
struct ConstructionCategoryTab(ConstructionCategory);
#[derive(Component)]
struct ConstructionItemListPanel;
#[derive(Component)]
struct ConstructionItemButton(GameBuildingType);
#[derive(Component)]
struct ConstructionItemDetailsPanel;
#[derive(Component)]
struct ConfirmBuildButton(GameBuildingType);
#[derive(Component)]
struct ConstructHabitationButton(usize); // usize is tier_index
#[derive(Component)]
struct ConstructServiceButton(ServiceType, usize); // ServiceType, tier_index
#[derive(Component)]
struct ConstructZoneButton(ZoneType, usize); // ZoneType, tier_index


// Colony Status Components
#[derive(Component, Debug, Clone, Copy)]
enum DiagnosticType {
    NutrientPaste,
    Housing,
    Healthcare,
    Security,
    Recreation,
    Education,
}
#[derive(Component)]
struct DiagnosticItem(DiagnosticType);


// Research Components
#[derive(Component)]
struct AvailableResearchListPanel;
#[derive(Component)]
struct ResearchItemButton(Tech);
#[derive(Component)]
struct ResearchDetailsPanel;
#[derive(Component)]
struct InitiateResearchButton;


// --- Resource & State Enums ---

#[derive(Resource, Default, Debug)]
pub struct SelectedBuilding(pub Option<GameBuildingType>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConstructionCategory {
    #[default]
    Operations,
    Habitation,
    Services,
    Zones,
}
#[derive(Resource, Default)]
pub struct CurrentConstructionCategory(pub ConstructionCategory);

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

#[derive(Resource, Default)]
pub struct SelectedTech(pub Option<Tech>);

#[derive(Resource, Default)]
pub struct SelectedZone(pub Option<String>);


// --- Building Metadata ---

#[derive(Clone, Copy, Debug)]
pub struct BuildingMetadata {
    pub name: &'static str,
    pub category: ConstructionCategory,
    pub required_tech: Option<Tech>,
    pub required_dp: Option<DevelopmentPhase>,
    pub workforce_required: u32,
}

fn get_building_metadata() -> HashMap<GameBuildingType, BuildingMetadata> {
    let mut meta = HashMap::new();
    meta.insert(GameBuildingType::Extractor, BuildingMetadata { name: "Extractor", category: ConstructionCategory::Operations, required_tech: None, required_dp: None, workforce_required: 5 });
    meta.insert(GameBuildingType::BioDome, BuildingMetadata { name: "Bio-Dome", category: ConstructionCategory::Operations, required_tech: None, required_dp: None, workforce_required: 10 });
    meta.insert(GameBuildingType::PowerRelay, BuildingMetadata { name: "Power Relay", category: ConstructionCategory::Operations, required_tech: None, required_dp: None, workforce_required: 0 });
    meta.insert(GameBuildingType::StorageSilo, BuildingMetadata { name: "Storage Silo", category: ConstructionCategory::Operations, required_tech: Some(Tech::BasicConstructionProtocols), required_dp: None, workforce_required: 0 });
    meta.insert(GameBuildingType::ResearchInstitute, BuildingMetadata { name: "Research Institute", category: ConstructionCategory::Operations, required_tech: Some(Tech::BasicConstructionProtocols), required_dp: None, workforce_required: 15 });
    meta.insert(GameBuildingType::Fabricator, BuildingMetadata { name: "Fabricator", category: ConstructionCategory::Operations, required_tech: Some(Tech::BasicConstructionProtocols), required_dp: None, workforce_required: 20 });
    meta.insert(GameBuildingType::ProcessingPlant, BuildingMetadata { name: "Processing Plant", category: ConstructionCategory::Operations, required_tech: Some(Tech::BasicConstructionProtocols), required_dp: None, workforce_required: 20 });
    meta
}


fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.1, 0.05, 0.15)))
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
        app.init_resource::<CurrentApp>()
            .init_resource::<CurrentConstructionCategory>()
            .init_resource::<SelectedBuilding>()
            .init_resource::<SelectedTech>()
            .init_resource::<SelectedZone>() // Initialize SelectedZone
            .add_systems(Startup, setup_ui)
            .add_systems(Update, (
                app_drawer_button_system,
                manage_app_panels_visibility,
                update_status_ticker_system,
                update_dashboard_notifications_system,
                update_admin_spire_panel_system,
                update_managed_structures_panel_system,
                zone_list_button_interaction_system,
                upgrade_zone_button_interaction_system,
                remove_zone_button_interaction_system,
                assign_specialist_to_zone_button_interaction_system, // Added new system
                unassign_specialist_from_zone_button_interaction_system, // Added new system
                admin_spire_button_interaction_system,
                draw_graph_gizmos,
                construction_category_tab_system,
                update_construction_list_system,
                construction_item_interaction_system,
                update_construction_details_panel_system,
                construction_interaction_system,
                habitation_construction_system,
                service_construction_system,
                zone_construction_system, // Added new system
                update_colony_status_panel_system,
                update_research_panel_system,
                research_item_button_system,
                update_research_details_panel_system,
                initiate_research_button_system,
            ).chain());
    }
}

fn zone_construction_system(
    mut interaction_query: Query<(&Interaction, &ConstructZoneButton), (Changed<Interaction>, With<Button>)>,
    mut game_state: ResMut<GameState>,
) {
    for (interaction, button_data) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            let zone_type = button_data.0;
            let tier_index = button_data.1;
            game_state::add_zone(&mut game_state, zone_type, tier_index);
            // Notifications will be handled in game_state::add_zone
        }
    }
}

fn service_construction_system(
    mut interaction_query: Query<(&Interaction, &ConstructServiceButton), (Changed<Interaction>, With<Button>)>,
    mut game_state: ResMut<GameState>,
) {
    for (interaction, button_data) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            let service_type = button_data.0;
            let tier_index = button_data.1;
            // Assuming add_service_building might take position as an Option in the future.
            // For now, it's not used as per game_state.rs, but good to keep in mind.
            game_state::add_service_building(&mut game_state, service_type, tier_index, None);
            // Consider adding a notification here if desired, similar to other construction systems
        }
    }
}


fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        ..default()
    })
    .with_children(|parent| {
        parent.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(8.0)),
                align_items: AlignItems::Center,
                flex_wrap: FlexWrap::Wrap,
                border: UiRect::bottom(Val::Px(2.0)),
                ..default()
            },
            background_color: PANEL_BACKGROUND.into(),
            border_color: BORDER_COLOR.into(),
            ..default()
        }).with_children(|ticker| {
            ticker.spawn((TextBundle::from_section("Cr.", TextStyle { font_size: 18.0, color: Color::GOLD, ..default() }).with_style(Style { margin: UiRect::horizontal(Val::Px(10.0)), ..default() }), CreditsText));
            ticker.spawn((TextBundle::from_section("⚡", TextStyle { font_size: 18.0, color: Color::CYAN, ..default() }).with_style(Style { margin: UiRect::horizontal(Val::Px(10.0)), ..default() }), PowerText));
            ticker.spawn((TextBundle::from_section("👤", TextStyle { font_size: 18.0, color: Color::WHITE, ..default() }).with_style(Style { margin: UiRect::horizontal(Val::Px(10.0)), ..default() }), PopulationText));
            ticker.spawn((TextBundle::from_section("🛠️", TextStyle { font_size: 18.0, color: Color::ORANGE, ..default() }).with_style(Style { margin: UiRect::horizontal(Val::Px(10.0)), ..default() }), WorkforceText));
            let core_resources = [ResourceType::NutrientPaste, ResourceType::FerrocreteOre, ResourceType::CuprumDeposits];
            for res in core_resources {
                ticker.spawn((TextBundle::from_section(format!("{:?}", res), TextStyle {font_size: 16.0, color: LABEL_TEXT_COLOR, ..default()}).with_style(Style { margin: UiRect::horizontal(Val::Px(10.0)), ..default() }), CoreResourceText(res)));
            }
            ticker.spawn((TextBundle::from_section("😊", TextStyle { font_size: 18.0, color: Color::GREEN, ..default() }).with_style(Style { margin: UiRect::horizontal(Val::Px(10.0)), ..default() }), ColonyHappinessText));
        });

        parent.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Row,
                ..default()
            },
            ..default()
        }).with_children(|main_area| {
            main_area.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(70.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(5.0)),
                    border: UiRect::right(Val::Px(2.0)),
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: PANEL_BACKGROUND.into(),
                border_color: BORDER_COLOR.into(),
                ..default()
            }).with_children(|drawer| {
                let apps = [
                    (AppType::Dashboard, "icon_dashboard.png"), 
                    (AppType::Construction, "icon_construction.png"),
                    (AppType::ColonyStatus, "icon_colony_status.png"),
                    (AppType::Research, "icon_research.png"),
                ];
                for (app_type, icon_path) in apps {
                    drawer.spawn((
                        ButtonBundle {
                            style: Style {
                                width: Val::Px(50.0),
                                height: Val::Px(50.0),
                                margin: UiRect::all(Val::Px(5.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            background_color: NORMAL_BUTTON.into(),
                            border_color: BORDER_COLOR.into(),
                            ..default()
                        },
                        AppDrawerButton(app_type),
                    )).with_children(|button| {
                        button.spawn(ImageBundle {
                            style: Style {
                                width: Val::Percent(70.0),
                                height: Val::Percent(70.0),
                                ..default()
                            },
                            image: UiImage::new(asset_server.load(icon_path)),
                            ..default()
                        });
                    });
                }
            });

            main_area.spawn(NodeBundle {
                style: Style {
                    flex_grow: 1.0,
                    height: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                ..default()
            }).with_children(|viewport| {

                viewport.spawn((NodeBundle { style: Style {display: Display::Flex, width: Val::Percent(100.0), height:Val::Percent(100.0), flex_direction: FlexDirection::Column, ..default()}, ..default() }, DashboardPanel))
                .with_children(|dash| {
                    dash.spawn(TextBundle::from_section("NEXUS DASHBOARD", TextStyle{font_size: 28.0, color: BORDER_COLOR, ..default()}).with_style(Style{margin: UiRect::bottom(Val::Px(10.0)), ..default()}));
                    
                    dash.spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Row,
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        ..default()
                    }).with_children(|main_dash_area| {
                        // Left side of dashboard
                        main_dash_area.spawn(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Column,
                                flex_grow: 1.0,
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            ..default()
                        }).with_children(|left_col| {
                            left_col.spawn((NodeBundle { style: Style {width: Val::Percent(100.0), height: Val::Percent(30.0), flex_direction: FlexDirection::Column, padding: UiRect::all(Val::Px(5.0)), border: UiRect::all(Val::Px(1.0)), ..default()}, background_color: Color::rgba(0.0,0.0,0.0,0.3).into(), border_color: BORDER_COLOR.into(), ..default()}, NotificationsPanel));
                            left_col.spawn((NodeBundle { style: Style {width: Val::Percent(100.0), flex_grow: 1.0, margin: UiRect::top(Val::Px(10.0)), border: UiRect::all(Val::Px(1.0)), ..default()}, background_color: Color::rgba(0.0,0.0,0.0,0.3).into(), border_color: BORDER_COLOR.into(), ..default()}, AnalyticsGraphPanel))
                            .with_children(|graph_panel| {
                                graph_panel.spawn((NodeBundle{style: Style{width: Val::Percent(100.0), height:Val::Percent(100.0), ..default()}, ..default()}, GraphArea));
                            });
                        });
                        // Right side of dashboard
                        main_dash_area.spawn(NodeBundle {
                             style: Style {
                                flex_direction: FlexDirection::Column,
                                width: Val::Px(300.0),
                                height: Val::Percent(100.0),
                                margin: UiRect::left(Val::Px(10.0)),
                                padding: UiRect::all(Val::Px(5.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                             },
                             background_color: Color::rgba(0.0,0.0,0.0,0.3).into(),
                             border_color: BORDER_COLOR.into(),
                             ..default()
                        }).with_children(|right_col| {
                            right_col.spawn(TextBundle::from_section("SYSTEM CONTROL", TextStyle{font_size: 18.0, color: LABEL_TEXT_COLOR, ..default()}));
                            right_col.spawn((NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Column,
                                    margin: UiRect::top(Val::Px(10.0)),
                                    padding: UiRect::all(Val::Px(5.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                border_color: BORDER_COLOR.into(),
                                ..default()
                            }, AdminSpireInfoPanel));

                            // Add ManagedStructuresPanel here
                            right_col.spawn((
                                NodeBundle {
                                    style: Style {
                                        flex_direction: FlexDirection::Column,
                                        margin: UiRect::top(Val::Px(10.0)),
                                        padding: UiRect::all(Val::Px(5.0)),
                                        border: UiRect::all(Val::Px(1.0)),
                                        flex_grow: 1.0, // Allow it to take available space
                                        ..default()
                                    },
                                    background_color: Color::rgba(0.0,0.0,0.0,0.2).into(), // Slightly different bg for distinction
                                    border_color: BORDER_COLOR.into(),
                                    ..default()
                                },
                                ManagedStructuresPanel,
                            ));
                        });
                    });
                });

                viewport.spawn((NodeBundle { style: Style {display: Display::None, width: Val::Percent(100.0), height:Val::Percent(100.0), flex_direction: FlexDirection::Column, ..default()}, ..default() }, ConstructionPanel))
                .with_children(|con| {
                    con.spawn(TextBundle::from_section("CONSTRUCTION", TextStyle{font_size: 28.0, color: BORDER_COLOR, ..default()}).with_style(Style{margin: UiRect::bottom(Val::Px(10.0)), ..default()}));
                    con.spawn(NodeBundle { style: Style { flex_direction: FlexDirection::Row, margin: UiRect::bottom(Val::Px(5.0)), ..default()}, ..default()})
                    .with_children(|tabs|{
                        let categories = [ConstructionCategory::Operations, ConstructionCategory::Habitation, ConstructionCategory::Services, ConstructionCategory::Zones];
                        for category in categories {
                            tabs.spawn((ButtonBundle {style: Style{padding: UiRect::all(Val::Px(8.0)), margin: UiRect::horizontal(Val::Px(5.0)), ..default()}, background_color: NORMAL_BUTTON.into(), ..default()}, ConstructionCategoryTab(category)))
                            .with_children(|button| { button.spawn(TextBundle::from_section(format!("{:?}", category), TextStyle {font_size: 16.0, color: PRIMARY_TEXT_COLOR, ..default()})); });
                        }
                    });
                    con.spawn(NodeBundle { style: Style {flex_direction: FlexDirection::Row, flex_grow: 1.0, ..default()}, ..default()})
                    .with_children(|main| {
                        main.spawn((NodeBundle{style: Style{width:Val::Percent(40.0), height: Val::Percent(100.0), border: UiRect::all(Val::Px(1.0)), padding: UiRect::all(Val::Px(5.0)), flex_direction: FlexDirection::Column, ..default()}, border_color: BORDER_COLOR.into(), ..default()}, ConstructionItemListPanel));
                        main.spawn((NodeBundle{style: Style{flex_grow: 1.0, height: Val::Percent(100.0), border: UiRect::all(Val::Px(1.0)), padding: UiRect::all(Val::Px(10.0)), margin: UiRect::left(Val::Px(10.0)), flex_direction:FlexDirection::Column, ..default()}, border_color: BORDER_COLOR.into(), ..default()}, ConstructionItemDetailsPanel));
                    });
                });
                
                viewport.spawn((NodeBundle { style: Style {display: Display::None, width: Val::Percent(100.0), height:Val::Percent(100.0), flex_direction: FlexDirection::Column, ..default()}, ..default() }, ColonyStatusPanel))
                .with_children(|status| {
                    status.spawn(TextBundle::from_section("COLONY STATUS", TextStyle{font_size: 28.0, color: BORDER_COLOR, ..default()}).with_style(Style{margin: UiRect::bottom(Val::Px(10.0)), ..default()}));
                    status.spawn(TextBundle::from_section("NEEDS DIAGNOSTIC", TextStyle{font_size: 20.0, color: LABEL_TEXT_COLOR, ..default()}).with_style(Style{margin: UiRect::bottom(Val::Px(10.0)), ..default()}));
                    let diagnostics = [DiagnosticType::NutrientPaste, DiagnosticType::Housing, DiagnosticType::Healthcare, DiagnosticType::Security, DiagnosticType::Recreation, DiagnosticType::Education];
                    for diag_type in diagnostics {
                        status.spawn((
                            TextBundle::from_section(format!("{:?}", diag_type), TextStyle{font_size: 18.0, color: PRIMARY_TEXT_COLOR, ..default()})
                                .with_style(Style{ margin: UiRect::bottom(Val::Px(5.0)), ..default()}),
                            DiagnosticItem(diag_type)
                        ));
                    }
                });

                viewport.spawn((NodeBundle { style: Style {display: Display::None, width: Val::Percent(100.0), height:Val::Percent(100.0), flex_direction: FlexDirection::Column, ..default()}, ..default() }, ResearchPanel))
                 .with_children(|research| {
                    research.spawn(TextBundle::from_section("RESEARCH & DEVELOPMENT", TextStyle{font_size: 28.0, color: BORDER_COLOR, ..default()}).with_style(Style{margin: UiRect::bottom(Val::Px(10.0)), ..default()}));
                     research.spawn(NodeBundle { style: Style {flex_direction: FlexDirection::Row, flex_grow: 1.0, ..default()}, ..default()})
                    .with_children(|main| {
                        main.spawn((NodeBundle{style: Style{width:Val::Percent(40.0), height: Val::Percent(100.0), border: UiRect::all(Val::Px(1.0)), padding: UiRect::all(Val::Px(5.0)), flex_direction: FlexDirection::Column, ..default()}, border_color: BORDER_COLOR.into(), ..default()}, AvailableResearchListPanel));
                        main.spawn((NodeBundle{style: Style{flex_grow: 1.0, height: Val::Percent(100.0), border: UiRect::all(Val::Px(1.0)), padding: UiRect::all(Val::Px(10.0)), margin: UiRect::left(Val::Px(10.0)), flex_direction:FlexDirection::Column, ..default()}, border_color: BORDER_COLOR.into(), ..default()}, ResearchDetailsPanel));
                    });
                });
            });
        });
    });
}

// --- UI UPDATE SYSTEMS ---

fn app_drawer_button_system(
    mut current_app_res: ResMut<CurrentApp>,
    mut button_query: Query<(&Interaction, &AppDrawerButton, &mut BackgroundColor), With<Button>>,
) {
    for (interaction, app_button, mut bg_color) in button_query.iter_mut() {
        if app_button.0 == current_app_res.0 {
            *bg_color = ACTIVE_BUTTON.into();
        } else {
            *bg_color = NORMAL_BUTTON.into();
        }

        if *interaction == Interaction::Pressed {
            if current_app_res.0 != app_button.0 {
                current_app_res.0 = app_button.0;
            }
        } else if *interaction == Interaction::Hovered && app_button.0 != current_app_res.0 {
            *bg_color = HOVERED_BUTTON.into();
        }
    }
}

fn update_managed_structures_panel_system(
    current_app: Res<CurrentApp>,
    game_state: Res<GameState>,
    selected_zone: Res<SelectedZone>,
    panel_query: Query<Entity, With<ManagedStructuresPanel>>,
    mut commands: Commands,
) {
    if current_app.0 != AppType::Dashboard {
        return;
    }

    // Rerun if game state, current app, or selected zone changes
    if game_state.is_changed() || (current_app.is_changed() && current_app.0 == AppType::Dashboard) || selected_zone.is_changed() {
        if let Ok(panel_entity) = panel_query.get_single() {
            commands.entity(panel_entity).despawn_descendants(); // Clear the whole panel

            commands.entity(panel_entity).with_children(|parent| {
                // --- Part 1: List of Zones (Buttons) ---
                parent.spawn(TextBundle::from_section(
                    "Managed Zones List", // Changed title for clarity
                    TextStyle {
                        font_size: 16.0, // Restoring original style for title
                        color: LABEL_TEXT_COLOR,
                        ..default()
                    },
                ).with_style(Style { margin: UiRect::bottom(Val::Px(5.0)), ..default() }));

                if game_state.zones.is_empty() {
                    parent.spawn(TextBundle::from_section(
                        "No zones established.",
                        TextStyle {
                            font_size: 14.0,
                            color: PRIMARY_TEXT_COLOR,
                            ..default()
                        },
                    ));
                } else {
                    for zone in game_state.zones.iter() {
                        if let Some(tier) = zone.available_tiers.get(zone.current_tier_index) {
                            parent.spawn((
                                ButtonBundle {
                                    style: Style {
                                        width: Val::Percent(100.0),
                                        padding: UiRect::all(Val::Px(5.0)),
                                        margin: UiRect::bottom(Val::Px(2.0)),
                                        justify_content: JustifyContent::Center, // Center text in button
                                        ..default()
                                    },
                                    background_color: NORMAL_BUTTON.into(),
                                    ..default()
                                },
                                ZoneListButton(zone.id.clone()),
                            ))
                            .with_children(|button_parent| {
                                button_parent.spawn(TextBundle::from_section(
                                    format!(
                                        "{:?} - {} (Workers: {}/{})",
                                        zone.zone_type,
                                        tier.name,
                                        zone.assigned_specialists,
                                        tier.specialist_jobs_provided
                                    ),
                    TextStyle {
                        font_size: 16.0,
                        color: LABEL_TEXT_COLOR,
                        ..default()
                    },
                ).with_style(Style { margin: UiRect::bottom(Val::Px(5.0)), ..default() }));

                if game_state.zones.is_empty() {
                    parent.spawn(TextBundle::from_section(
                        "No zones established.",
                        TextStyle {
                            font_size: 14.0,
                            color: PRIMARY_TEXT_COLOR,
                            ..default()
                        },
                    ));
                } else {
                    for zone in game_state.zones.iter() {
                        if let Some(tier) = zone.available_tiers.get(zone.current_tier_index) {
                            parent.spawn((
                                ButtonBundle {
                                    style: Style {
                                        width: Val::Percent(100.0),
                                        padding: UiRect::all(Val::Px(5.0)),
                                        margin: UiRect::bottom(Val::Px(2.0)),
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    background_color: NORMAL_BUTTON.into(), // This will be updated by zone_list_button_interaction_system
                                    ..default()
                                },
                                ZoneListButton(zone.id.clone()),
                            ))
                            .with_children(|button_parent| {
                                button_parent.spawn(TextBundle::from_section(
                                    format!(
                                        "{:?} - {} (Workers: {}/{})",
                                        zone.zone_type,
                                        tier.name,
                                        zone.assigned_specialists,
                                        tier.specialist_jobs_provided
                                    ),
                                    TextStyle {
                                        font_size: 14.0,
                                        color: PRIMARY_TEXT_COLOR,
                                        ..default()
                                    },
                                ));
                            });
                        }
                    }
                }

                // --- Part 2: Details of Selected Zone ---
                // Unconditionally spawn the ZoneDetailsPanel container, then conditionally populate it.
                let details_panel_entity = parent.spawn((
                    NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            margin: UiRect::top(Val::Px(10.0)),
                            padding: UiRect::all(Val::Px(5.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            border_color: Color::rgba(0.5, 0.5, 0.5, 0.5).into(),
                            min_height: Val::Px(150.0), // Ensure it has some space
                            ..default()
                        },
                        background_color: Color::rgba(0.1, 0.1, 0.1, 0.3).into(),
                        ..default()
                    },
                    ZoneDetailsPanel,
                )).id();

                if let Some(selected_zone_id) = &selected_zone.0 {
                    if let Some(zone) = game_state.zones.iter().find(|z| z.id == *selected_zone_id) {
                        if let Some(current_tier) = zone.available_tiers.get(zone.current_tier_index) {
                            commands.entity(details_panel_entity).with_children(|details_parent| {
                                details_parent.spawn(TextBundle::from_section(
                                    format!("Zone Details: {:?} - {}", zone.zone_type, current_tier.name),
                                    TextStyle { font_size: 15.0, color: PRIMARY_TEXT_COLOR, ..default()}
                                ).with_style(Style{ margin: UiRect::bottom(Val::Px(4.0)), ..default()}));

                                details_parent.spawn(TextBundle::from_section(
                                    format!("Workers: {}/{}", zone.assigned_specialists, current_tier.specialist_jobs_provided),
                                    TextStyle { font_size: 14.0, color: LABEL_TEXT_COLOR, ..default()}
                                ).with_style(Style{ margin: UiRect::bottom(Val::Px(2.0)), ..default()}));

                                details_parent.spawn(TextBundle::from_section(
                                    format!("Civic Index: {}", current_tier.civic_index_contribution),
                                    TextStyle { font_size: 14.0, color: LABEL_TEXT_COLOR, ..default()}
                                ).with_style(Style{ margin: UiRect::bottom(Val::Px(2.0)), ..default()}));

                                if zone.zone_type == ZoneType::Commercial {
                                    details_parent.spawn(TextBundle::from_section(
                                        format!("Income: {} Cr/cycle", current_tier.income_generation),
                                        TextStyle { font_size: 14.0, color: LABEL_TEXT_COLOR, ..default()}
                                    ).with_style(Style{ margin: UiRect::bottom(Val::Px(2.0)), ..default()}));
                                }

                                details_parent.spawn(TextBundle::from_section(
                                    format!("Upkeep: {} Cr/cycle", current_tier.upkeep_cost),
                                    TextStyle { font_size: 14.0, color: LABEL_TEXT_COLOR, ..default()}
                                ).with_style(Style{ margin: UiRect::bottom(Val::Px(8.0)), ..default()}));

                                // Upgrade Button
                                if zone.current_tier_index < zone.available_tiers.len() - 1 {
                                    let next_tier = &zone.available_tiers[zone.current_tier_index + 1];
                                    let can_afford_upgrade = game_state.credits >= next_tier.construction_credits_cost as f64;
                                    details_parent.spawn((
                                        ButtonBundle {
                                            style: Style { width: Val::Percent(100.0), padding: UiRect::all(Val::Px(5.0)), margin: UiRect::bottom(Val::Px(4.0)), ..default()},
                                            background_color: if can_afford_upgrade { NORMAL_BUTTON.into() } else { DISABLED_BUTTON.into() },
                                            ..default()
                                        },
                                        UpgradeZoneButton(selected_zone_id.clone()),
                                    )).with_children(|btn| {
                                        btn.spawn(TextBundle::from_section(
                                            format!("Upgrade to {} ({} Cr)", next_tier.name, next_tier.construction_credits_cost),
                                            TextStyle { font_size: 14.0, color: PRIMARY_TEXT_COLOR, ..default()}
                                        ));
                                    });
                                } else {
                                     details_parent.spawn(TextBundle::from_section( "Max tier reached", TextStyle { font_size: 14.0, color: Color::CYAN, ..default()}));
                                }

                                // Remove Button
                                details_parent.spawn((
                                    ButtonBundle {
                                        style: Style { width: Val::Percent(100.0), padding: UiRect::all(Val::Px(5.0)), margin: UiRect::top(Val::Px(4.0)), ..default()},
                                        background_color: NORMAL_BUTTON.into(), // Or a more "dangerous" color
                                        ..default()
                                    },
                                    RemoveZoneButton(selected_zone_id.clone()),
                                )).with_children(|btn| {
                                    btn.spawn(TextBundle::from_section("Remove Zone", TextStyle { font_size: 14.0, color: Color::TOMATO, ..default()}));
                                });

                                // Spacer before specialist buttons
                                details_parent.spawn(NodeBundle { style: Style { height: Val::Px(10.0), ..default()}, ..default()});

                                // Assign Specialist Button
                                let can_assign_more_specialists_to_zone = zone.assigned_specialists < current_tier.specialist_jobs_provided;
                                let available_general_inhabitants = game_state.total_inhabitants.saturating_sub(game_state.assigned_specialists_total);
                                let can_assign = can_assign_more_specialists_to_zone && available_general_inhabitants > 0;

                                details_parent.spawn((
                                    ButtonBundle {
                                        style: Style { width: Val::Percent(100.0), padding: UiRect::all(Val::Px(5.0)), margin: UiRect::bottom(Val::Px(4.0)), ..default()},
                                        background_color: if can_assign { NORMAL_BUTTON.into() } else { DISABLED_BUTTON.into() },
                                        ..default()
                                    },
                                    AssignSpecialistToZoneButton(selected_zone_id.clone()),
                                )).with_children(|btn| {
                                    btn.spawn(TextBundle::from_section("Assign Specialist (+1)", TextStyle { font_size: 14.0, color: PRIMARY_TEXT_COLOR, ..default()}));
                                });

                                // Unassign Specialist Button
                                let can_unassign = zone.assigned_specialists > 0;
                                details_parent.spawn((
                                    ButtonBundle {
                                        style: Style { width: Val::Percent(100.0), padding: UiRect::all(Val::Px(5.0)), ..default()}, // Removed bottom margin for last button
                                        background_color: if can_unassign { NORMAL_BUTTON.into() } else { DISABLED_BUTTON.into() },
                                        ..default()
                                    },
                                    UnassignSpecialistFromZoneButton(selected_zone_id.clone()),
                                )).with_children(|btn| {
                                    btn.spawn(TextBundle::from_section("Unassign Specialist (-1)", TextStyle { font_size: 14.0, color: PRIMARY_TEXT_COLOR, ..default()}));
                                });
                            });
                        }
                    }
                }
            });
        }
    }
}

fn assign_specialist_to_zone_button_interaction_system(
    mut interaction_query: Query<(&Interaction, &AssignSpecialistToZoneButton), (Changed<Interaction>, With<Button>)>,
    mut game_state: ResMut<GameState>,
) {
    for (interaction, button) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            // Basic check if button was active, game_state function will do the full logic check
            let zone_id = &button.0;
            if let Some(zone) = game_state.zones.iter().find(|z| z.id == *zone_id) {
                if let Some(tier) = zone.available_tiers.get(zone.current_tier_index){
                     let can_assign_more_specialists_to_zone = zone.assigned_specialists < tier.specialist_jobs_provided;
                     let available_general_inhabitants = game_state.total_inhabitants.saturating_sub(game_state.assigned_specialists_total);
                     if can_assign_more_specialists_to_zone && available_general_inhabitants > 0 {
                        game_state::assign_specialists_to_zone(&mut game_state, zone_id, 1);
                     }
                }
            }
        }
    }
}

fn unassign_specialist_from_zone_button_interaction_system(
    mut interaction_query: Query<(&Interaction, &UnassignSpecialistFromZoneButton), (Changed<Interaction>, With<Button>)>,
    mut game_state: ResMut<GameState>,
) {
    for (interaction, button) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            let zone_id = &button.0;
             if let Some(zone) = game_state.zones.iter().find(|z| z.id == *zone_id) {
                if zone.assigned_specialists > 0 {
                    game_state::unassign_specialists_from_zone(&mut game_state, zone_id, 1);
                }
            }
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
    if !current_app.is_changed() { return; }

    for mut style in panel_queries.p0().iter_mut() { style.display = Display::None; }
    for mut style in panel_queries.p1().iter_mut() { style.display = Display::None; }
    for mut style in panel_queries.p2().iter_mut() { style.display = Display::None; }
    for mut style in panel_queries.p3().iter_mut() { style.display = Display::None; }

    match current_app.0 {
        AppType::Dashboard => panel_queries.p0().single_mut().display = Display::Flex,
        AppType::Construction => panel_queries.p1().single_mut().display = Display::Flex,
        AppType::ColonyStatus => panel_queries.p2().single_mut().display = Display::Flex,
        AppType::Research => panel_queries.p3().single_mut().display = Display::Flex,
    }
}

fn update_status_ticker_system(
    game_state: Res<GameState>,
    mut queries: ParamSet<(
        Query<&mut Text, With<CreditsText>>,
        Query<&mut Text, With<PowerText>>,
        Query<&mut Text, With<PopulationText>>,
        Query<&mut Text, With<WorkforceText>>,
        Query<(&mut Text, &CoreResourceText)>,
        Query<&mut Text, With<ColonyHappinessText>>,
    )>,
) {
    // Credits
    queries.p0().single_mut().sections[0].value = format!("Cr. {:.0}", game_state.credits);
    
    // Power
    let net_power = game_state.total_generated_power - game_state.total_consumed_power;
    let stored_power = *game_state.current_resources.get(&ResourceType::Power).unwrap_or(&0.0);
    let mut p1 = queries.p1();
    let mut power_text = p1.single_mut();
    power_text.sections[0].value = format!("⚡ {:+.0} | 🔋 {:.0}", net_power, stored_power);
    power_text.sections[0].style.color = if net_power < 0.0 { Color::RED } else { Color::CYAN };
    
    // Population
    queries.p2().single_mut().sections[0].value = format!("👤 {} / {}", game_state.total_inhabitants, game_state.available_housing_capacity);

    // Workforce
    queries.p3().single_mut().sections[0].value = format!("🛠️ {} / {}", game_state.assigned_workforce, game_state.total_inhabitants);
    
    // Core Resources
    for (mut text, marker) in queries.p4().iter_mut() {
        let amount = game_state.current_resources.get(&marker.0).unwrap_or(&0.0);
        text.sections[0].value = format!("{:?}: {:.0}", marker.0, amount);
    }
    
    // Happiness
    let mut p5 = queries.p5();
    let mut happiness_text = p5.single_mut();
    happiness_text.sections[0].value = format!(
        "{} {:.0}%",
        match game_state.colony_happiness {
            h if h >= 85.0 => "😊", h if h >= 50.0 => "😐", _ => "☹️",
        },
        game_state.colony_happiness
    );
    happiness_text.sections[0].style.color = match game_state.colony_happiness {
            h if h >= 85.0 => Color::GREEN, h if h >= 50.0 => Color::YELLOW, _ => Color::RED,
    };
}


fn update_dashboard_notifications_system(
    current_app: Res<CurrentApp>,
    game_state: Res<GameState>,
    notifications_panel_query: Query<Entity, With<NotificationsPanel>>,
    mut commands: Commands,
) {
    if current_app.0 != AppType::Dashboard { return; }

    if game_state.is_changed() || current_app.is_changed() {
        if let Ok(panel_entity) = notifications_panel_query.get_single() {
            commands.entity(panel_entity).despawn_descendants();
            commands.entity(panel_entity).with_children(|parent| {
                parent.spawn(TextBundle::from_section("EVENT LOG", TextStyle{font_size: 18.0, color: LABEL_TEXT_COLOR, ..default()}));
                for event in game_state.notifications.iter().take(5) {
                    parent.spawn(TextBundle::from_section(
                        format!("[{:.1}] {}", event.timestamp, event.message),
                        TextStyle { font_size: 14.0, color: PRIMARY_TEXT_COLOR, ..default() }
                    ).with_style(Style{ margin: UiRect::top(Val::Px(4.0)), ..default()}));
                }
            });
        }
    }
}

fn update_admin_spire_panel_system(
    game_state: Res<GameState>,
    panel_query: Query<Entity, With<AdminSpireInfoPanel>>,
    mut commands: Commands,
) {
    if !game_state.is_changed() { return; }

    if let Ok(panel_entity) = panel_query.get_single() {
        commands.entity(panel_entity).despawn_descendants();

        commands.entity(panel_entity).with_children(|parent| {
            parent.spawn(TextBundle::from_section("Administrative Spire", TextStyle{font_size: 16.0, color: PRIMARY_TEXT_COLOR, ..default()}));

            if let Some(spire) = &game_state.administrative_spire {
                let current_tier = &spire.available_tiers[spire.current_tier_index];
                parent.spawn((
                    TextBundle::from_section(format!("Tier: {}", current_tier.name), TextStyle{font_size: 14.0, color: LABEL_TEXT_COLOR, ..default()}),
                    AdminSpireTierText,
                ));
                 parent.spawn(TextBundle::from_section(format!("Phase: {:?}", game_state.current_development_phase), TextStyle{font_size: 14.0, color: LABEL_TEXT_COLOR, ..default()}));

                if spire.current_tier_index < spire.available_tiers.len() - 1 {
                    let next_tier = &spire.available_tiers[spire.current_tier_index + 1];
                    let can_afford = game_state.credits >= next_tier.upgrade_credits_cost as f64;
                    parent.spawn((
                        ButtonBundle {
                            style: Style { width: Val::Percent(100.0), padding: UiRect::all(Val::Px(5.0)), margin: UiRect::top(Val::Px(10.0)), ..default()},
                            background_color: if can_afford { NORMAL_BUTTON.into() } else { DISABLED_BUTTON.into() },
                            ..default()
                        },
                        UpgradeSpireButton
                    )).with_children(|btn| {
                        btn.spawn(TextBundle::from_section(
                            format!("Upgrade to {}\n({} Cr)", next_tier.name, next_tier.upgrade_credits_cost), 
                            TextStyle{font_size: 14.0, color: PRIMARY_TEXT_COLOR, ..default()}
                        ));
                    });
                } else {
                    parent.spawn(TextBundle::from_section("Max Tier Reached", TextStyle{font_size: 14.0, color: Color::CYAN, ..default()}));
                }
            } else {
                 parent.spawn((
                    TextBundle::from_section("Status: Not Constructed", TextStyle{font_size: 14.0, color: LABEL_TEXT_COLOR, ..default()}),
                    AdminSpireTierText
                ));
                let can_afford = game_state.credits >= 1000.0; // Hardcoded from game_state
                 parent.spawn((
                    ButtonBundle {
                        style: Style { width: Val::Percent(100.0), padding: UiRect::all(Val::Px(5.0)), margin: UiRect::top(Val::Px(10.0)), ..default()},
                        background_color: if can_afford { NORMAL_BUTTON.into() } else { DISABLED_BUTTON.into() },
                        ..default()
                    },
                    ConstructSpireButton
                )).with_children(|btn| {
                    btn.spawn(TextBundle::from_section(
                        "Construct (1000 Cr)",
                        TextStyle{font_size: 14.0, color: PRIMARY_TEXT_COLOR, ..default()}
                    ));
                });
            }
        });
    }
}

fn zone_list_button_interaction_system(
    mut selected_zone_res: ResMut<SelectedZone>,
    mut button_query: Query<(&Interaction, &ZoneListButton, &mut BackgroundColor), With<Button>>,
) {
    for (interaction, zone_button_data, mut bg_color) in button_query.iter_mut() {
        let is_currently_selected_in_res = selected_zone_res.0.as_ref() == Some(&zone_button_data.0);

        if *interaction == Interaction::Pressed {
            if is_currently_selected_in_res {
                selected_zone_res.0 = None;
            } else {
                selected_zone_res.0 = Some(zone_button_data.0.clone());
            }
        }

        if selected_zone_res.is_changed() || *interaction != Interaction::None { // Update if selection changed or interaction happened
            if selected_zone_res.0.as_ref() == Some(&zone_button_data.0) {
                *bg_color = ACTIVE_BUTTON.into();
            } else if *interaction == Interaction::Hovered {
                *bg_color = HOVERED_BUTTON.into();
            } else {
                *bg_color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn upgrade_zone_button_interaction_system(
    mut interaction_query: Query<(&Interaction, &UpgradeZoneButton), (Changed<Interaction>, With<Button>)>,
    mut game_state: ResMut<GameState>,
) {
    for (interaction, button) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            // Affordability is checked visually by button color in update_managed_structures_panel_system.
            // Direct click interaction here assumes if it's clickable (not explicitly blocked by Bevy's interaction system itself),
            // the action should be attempted. game_state::upgrade_zone should handle the actual logic including costs.
            game_state::upgrade_zone(&mut game_state, &button.0);
        }
    }
}

fn remove_zone_button_interaction_system(
    mut interaction_query: Query<(&Interaction, &RemoveZoneButton), (Changed<Interaction>, With<Button>)>,
    mut game_state: ResMut<GameState>,
    mut selected_zone: ResMut<SelectedZone>,
) {
    for (interaction, button) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            game_state::remove_zone(&mut game_state, &button.0);
            selected_zone.0 = None;
        }
    }
}

fn admin_spire_button_interaction_system(
    mut game_state: ResMut<GameState>,
    mut interaction_query: ParamSet<(
        Query<&Interaction, (Changed<Interaction>, With<ConstructSpireButton>)>,
        Query<&Interaction, (Changed<Interaction>, With<UpgradeSpireButton>)>,
    )>,
) {
    if let Ok(Interaction::Pressed) = interaction_query.p0().get_single() {
        game_state::construct_administrative_spire(&mut game_state);
    }

    if let Ok(Interaction::Pressed) = interaction_query.p1().get_single() {
        game_state::upgrade_administrative_spire(&mut game_state);
    }
}


trait GraphableFn: Fn(&ColonyStats) -> f32 + Send + Sync {}
impl<F: Fn(&ColonyStats) -> f32 + Send + Sync> GraphableFn for F {}

fn draw_graph_gizmos(
    mut gizmos: Gizmos,
    graph_data: Res<GraphData>,
    graph_area_query: Query<(&Node, &GlobalTransform), With<GraphArea>>,
) {
     if graph_data.history.is_empty() { return; }

    let (graph_node, transform) = if let Ok(result) = graph_area_query.get_single() { result } else { return; };
    let graph_area = graph_node.size();
    if graph_area.x <= 0.0 || graph_area.y <= 0.0 { return; }
    
    let bottom_left = transform.translation().truncate() - graph_area / 2.0;
    let max_val = graph_data.history.iter().fold(1.0f32, |max, stats| max.max(stats.credits as f32).max(stats.happiness).max(stats.net_power.abs()));

    let graph_lines: [(Color, Box<dyn GraphableFn>); 3] = [
        (Color::GOLD, Box::new(|stats| stats.credits as f32)),
        (Color::LIME_GREEN, Box::new(|stats| stats.happiness)),
        (Color::CYAN, Box::new(|stats| stats.net_power)),
    ];

    for (color, get_value) in graph_lines.iter() {
        let points: Vec<Vec2> = graph_data.history.iter().enumerate().map(|(i, stats)| {
            let x = bottom_left.x + (i as f32 / (graph_data.history.len() - 1).max(1) as f32) * graph_area.x;
            let y_val = get_value(stats);
            let y_normalized = y_val / max_val;
            let y = bottom_left.y + (y_normalized * 0.5 + 0.5) * graph_area.y;

            Vec2::new(x, y.clamp(bottom_left.y, bottom_left.y + graph_area.y))
        }).collect();

        if points.len() > 1 {
            gizmos.linestrip_2d(points, *color);
        }
    }
    gizmos.line_2d(
        Vec2::new(bottom_left.x, bottom_left.y + graph_area.y / 2.0),
        Vec2::new(bottom_left.x + graph_area.x, bottom_left.y + graph_area.y / 2.0),
        Color::rgba(1.0, 1.0, 1.0, 0.2),
    );
}

fn construction_category_tab_system(
    mut current_category: ResMut<CurrentConstructionCategory>,
    mut button_query: Query<(&Interaction, &ConstructionCategoryTab, &mut BackgroundColor), With<Button>>,
) {
    for (interaction, tab, mut bg_color) in button_query.iter_mut() {
        if tab.0 == current_category.0 { *bg_color = ACTIVE_BUTTON.into(); } else { *bg_color = NORMAL_BUTTON.into(); }

        if *interaction == Interaction::Pressed {
             if current_category.0 != tab.0 { current_category.0 = tab.0; }
        } else if *interaction == Interaction::Hovered && tab.0 != current_category.0 {
            *bg_color = HOVERED_BUTTON.into();
        }
    }
}

fn update_construction_list_system(
    current_app: Res<CurrentApp>,
    current_category: Res<CurrentConstructionCategory>,
    game_state: Res<GameState>,
    mut item_list_panel_query: Query<Entity, With<ConstructionItemListPanel>>,
    mut commands: Commands,
) {
    if current_app.0 != AppType::Construction { return; }

    if current_category.is_changed() || (current_app.is_changed() && current_app.0 == AppType::Construction) || game_state.is_changed() {
        if let Ok(panel_entity) = item_list_panel_query.get_single_mut() {
            commands.entity(panel_entity).despawn_descendants(); 

            commands.entity(panel_entity).with_children(|parent| {
                match current_category.0 {
                    ConstructionCategory::Operations => {
                        let building_meta_map = get_building_metadata();
                        let items: Vec<_> = ALL_BUILDING_TYPES.iter()
                            .filter_map(|bt| building_meta_map.get(bt).map(|meta| (bt, meta)))
                            .filter(|(_, meta)| meta.category == current_category.0)
                            .filter(|(_, meta)| {
                                meta.required_dp.is_none() || meta.required_dp.unwrap() <= game_state.current_development_phase
                            })
                            .collect();

                        if items.is_empty() {
                            parent.spawn(TextBundle::from_section("No items in this category.", TextStyle{color: LABEL_TEXT_COLOR, ..default()}));
                            return;
                        }

                        for (building_type, meta) in items {
                            parent.spawn((
                                ButtonBundle { style: Style { width: Val::Percent(100.0), padding: UiRect::all(Val::Px(8.0)), margin: UiRect::bottom(Val::Px(4.0)), ..default() }, background_color: NORMAL_BUTTON.into(), ..default()},
                                ConstructionItemButton(*building_type) 
                            )).with_children(|p| {
                                p.spawn(TextBundle::from_section(meta.name, TextStyle { font_size: 16.0, color: PRIMARY_TEXT_COLOR, ..default() }));
                            });
                        }
                    },
                    ConstructionCategory::Habitation => {
                        let habitation_tiers = game_state::get_habitation_tiers();
                        for (tier_index, tier) in habitation_tiers.iter().enumerate() {
                            let can_afford = game_state.credits >= tier.construction_credits_cost as f64;
                            parent.spawn((
                                ButtonBundle {
                                    style: Style { width: Val::Percent(100.0), padding: UiRect::all(Val::Px(8.0)), margin: UiRect::bottom(Val::Px(4.0)), ..default() },
                                    background_color: if can_afford { NORMAL_BUTTON.into() } else { DISABLED_BUTTON.into() },
                                    ..default()
                                },
                                ConstructHabitationButton(tier_index)
                            )).with_children(|p| {
                                p.spawn(TextBundle::from_section(
                                    format!("{} ({} Cr)", tier.name, tier.construction_credits_cost), 
                                    TextStyle { font_size: 16.0, color: PRIMARY_TEXT_COLOR, ..default() }
                                ));
                            });
                        }
                    },
                    ConstructionCategory::Services => {
                        let service_types = [
                            ServiceType::Wellness,
                            ServiceType::Security,
                            ServiceType::Education,
                            ServiceType::Recreation,
                            ServiceType::Spiritual,
                        ];

                        for service_type in service_types.iter() {
                            let service_tiers = game_state::get_service_building_tiers(*service_type);
                            if service_tiers.is_empty() {
                                // This case should ideally not happen if services always have tiers
                                parent.spawn(TextBundle::from_section(
                                    format!("No tiers available for {:?}.", service_type),
                                    TextStyle { color: LABEL_TEXT_COLOR, ..default() },
                                ));
                                continue;
                            }

                            for (tier_index, tier) in service_tiers.iter().enumerate() {
                                // TODO: Replace with actual ConstructServiceButton(service_type, tier_index)
                                // For now, create a generic button, actual component will be added later.
                                let can_afford = game_state.credits >= tier.construction_credits_cost as f64;
                                parent.spawn((
                                    ButtonBundle {
                                        style: Style {
                                            width: Val::Percent(100.0),
                                            padding: UiRect::all(Val::Px(8.0)),
                                            margin: UiRect::bottom(Val::Px(4.0)),
                                            ..default()
                                        },
                                        background_color: if can_afford { NORMAL_BUTTON.into() } else { DISABLED_BUTTON.into() },
                                        ..default()
                                    },
                                    ConstructServiceButton(*service_type, tier_index)
                                )).with_children(|p| {
                                    p.spawn(TextBundle::from_section(
                                        format!("{:?} - {} ({} Cr)", service_type, tier.name, tier.construction_credits_cost),
                                        TextStyle { font_size: 16.0, color: PRIMARY_TEXT_COLOR, ..default() }
                                    ));
                                });
                            }
                        }
                    },
                    ConstructionCategory::Zones => {
                        let zone_types = [ZoneType::Commercial, ZoneType::LightIndustry];

                        for zone_type in zone_types.iter() {
                            let zone_tiers = game_state::get_zone_tiers(*zone_type);
                            if zone_tiers.is_empty() {
                                parent.spawn(TextBundle::from_section(
                                    format!("No tiers available for {:?}.", zone_type),
                                    TextStyle { color: LABEL_TEXT_COLOR, ..default() },
                                ));
                                continue;
                            }

                            for (tier_index, tier) in zone_tiers.iter().enumerate() {
                                let can_afford = game_state.credits >= tier.construction_credits_cost as f64;
                                parent.spawn((
                                    ButtonBundle {
                                        style: Style {
                                            width: Val::Percent(100.0),
                                            padding: UiRect::all(Val::Px(8.0)),
                                            margin: UiRect::bottom(Val::Px(4.0)),
                                            ..default()
                                        },
                                        background_color: if can_afford { NORMAL_BUTTON.into() } else { DISABLED_BUTTON.into() },
                                        ..default()
                                    },
                                    ConstructZoneButton(*zone_type, tier_index)
                                )).with_children(|p| {
                                    p.spawn(TextBundle::from_section(
                                        format!("{:?} - {} ({} Cr)", zone_type, tier.name, tier.construction_credits_cost),
                                        TextStyle { font_size: 16.0, color: PRIMARY_TEXT_COLOR, ..default() }
                                    ));
                                });
                            }
                        }
                    }
                }
            });
        }
    }
}


fn habitation_construction_system(
    mut interaction_query: Query<(&Interaction, &ConstructHabitationButton)>,
    mut game_state: ResMut<GameState>,
) {
    for (interaction, button) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            let tier_index = button.0;
            game_state::add_habitation_structure(&mut game_state, tier_index);
        }
    }
}


fn construction_item_interaction_system(
    mut selected_building_res: ResMut<SelectedBuilding>,
    game_state: Res<GameState>, 
    mut button_query: Query<(&Interaction, &ConstructionItemButton, &mut BackgroundColor), With<Button>>,
) {
    let can_afford = |bt: GameBuildingType| -> bool {
         game_state.building_costs.get(&bt).map_or(true, |costs| {
            costs.iter().all(|(res, &req)| game_state.current_resources.get(res).unwrap_or(&0.0) >= &req)
        })
    };
    
    for (interaction, item_button, mut bg_color) in button_query.iter_mut() {
        let building_type = item_button.0;
        let is_selected = selected_building_res.0 == Some(building_type);

        if is_selected { *bg_color = ACTIVE_BUTTON.into(); } 
        else if !can_afford(building_type) { *bg_color = DISABLED_BUTTON.into(); }
        else { *bg_color = NORMAL_BUTTON.into(); }

        if *interaction == Interaction::Pressed {
            selected_building_res.0 = if is_selected { None } else { Some(building_type) };
        } else if *interaction == Interaction::Hovered && !is_selected && can_afford(building_type) {
             *bg_color = HOVERED_BUTTON.into();
        }
    }
}

fn update_construction_details_panel_system(
    selected_building: Res<SelectedBuilding>,
    game_state: Res<GameState>,
    mut details_panel_query: Query<Entity, With<ConstructionItemDetailsPanel>>,
    mut commands: Commands,
) {
    if !selected_building.is_changed() { return; }

    if let Ok(panel_entity) = details_panel_query.get_single_mut() {
        commands.entity(panel_entity).despawn_descendants();
        let building_meta_map = get_building_metadata();

        if let Some(building_type) = selected_building.0 {
             if let Some(meta) = building_meta_map.get(&building_type) {
                commands.entity(panel_entity).with_children(|parent| {
                    parent.spawn(TextBundle::from_section(meta.name, TextStyle{font_size: 22.0, color: PRIMARY_TEXT_COLOR, ..default()}));
                    
                    parent.spawn(TextBundle::from_section(format!("- Workforce Required: {}", meta.workforce_required), TextStyle{ color: LABEL_TEXT_COLOR, ..default()}));
                    
                    if let Some(costs) = game_state.building_costs.get(&building_type) {
                        for (res, amount) in costs {
                            parent.spawn(TextBundle::from_section(format!("- {:?}: {}", res, amount), TextStyle{ color: LABEL_TEXT_COLOR, ..default()}));
                        }
                    }
                     if let Some(tech) = meta.required_tech {
                         parent.spawn(TextBundle::from_section(format!("Req: {:?}", tech), TextStyle{ color: LABEL_TEXT_COLOR, ..default()}));
                     }
                    parent.spawn((ButtonBundle{style:Style{position_type: PositionType::Absolute, bottom: Val::Px(10.0), right: Val::Px(10.0), padding: UiRect::all(Val::Px(10.0)), ..default()}, background_color: NORMAL_BUTTON.into(), ..default()}, ConfirmBuildButton(building_type)))
                    .with_children(|button| { button.spawn(TextBundle::from_section("CONSTRUCT", TextStyle{color: PRIMARY_TEXT_COLOR, ..default()})); });
                });
            }
        } else {
            commands.entity(panel_entity).with_children(|parent| {
                parent.spawn(TextBundle::from_section("Select an item to view details.", TextStyle { color: LABEL_TEXT_COLOR, ..default() }));
            });
        }
    }
}

fn construction_interaction_system(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ConfirmBuildButton>)>,
    selected_building: Res<SelectedBuilding>,
    mut game_state: ResMut<GameState>,
    mut commands: Commands,
    time: Res<Time>,
) {
     if let Some(building_type) = selected_building.0 {
        if let Ok(Interaction::Pressed) = interaction_query.get_single() {
             let meta = get_building_metadata().get(&building_type).unwrap().clone();
             let costs = game_state.building_costs.get(&building_type).unwrap().clone();
             
             let can_afford = costs.iter().all(|(res, &req)| game_state.current_resources.get(res).unwrap_or(&0.0) >= &req);
             if !can_afford {
                 game_state::add_notification(&mut game_state.notifications, format!("Insufficient materials for {:?}.", building_type), time.elapsed_seconds_f64());
                 return;
             }

             for (res, cost) in &costs { 
                 *game_state.current_resources.get_mut(res).unwrap() -= cost; 
             }
             
             match building_type {
                GameBuildingType::Extractor => { commands.spawn(game_state::Extractor { power_consumption: 15, resource_type: ResourceType::FerrocreteOre, extraction_rate: 2.5, workforce_required: meta.workforce_required, is_staffed: false }); }
                GameBuildingType::BioDome => { commands.spawn(game_state::BioDome { power_consumption: 10, production_rate: 5.0, workforce_required: meta.workforce_required, is_staffed: false }); }
                GameBuildingType::PowerRelay => { commands.spawn(game_state::PowerRelay { power_output: 50 }); }
                GameBuildingType::ResearchInstitute => { commands.spawn(game_state::ResearchInstitute { power_consumption: 5, workforce_required: meta.workforce_required, is_staffed: false }); }
                GameBuildingType::Fabricator => {
                    game_state::add_fabricator(&mut game_state, 0); 
                }
                GameBuildingType::ProcessingPlant => {
                    game_state::add_processing_plant(&mut game_state, 0);
                }
                GameBuildingType::StorageSilo => {
                    commands.spawn(game_state::StorageSilo { capacity: 500 });
                }
             }
             game_state::add_notification(&mut game_state.notifications, format!("Construction started: {:?}", building_type), time.elapsed_seconds_f64());
        }
     }
}


fn update_colony_status_panel_system(
    game_state: Res<GameState>,
    mut query: Query<(&mut Text, &DiagnosticItem)>,
) {
    if !game_state.is_changed() { return; }

    for (mut text, item) in query.iter_mut() {
        let (status_text, color) = match item.0 {
            DiagnosticType::NutrientPaste => {
                let status = game_state.simulated_has_sufficient_nutrient_paste;
                (if status { "Surplus" } else { "Deficit" }, if status { Color::GREEN } else { Color::RED })
            },
            DiagnosticType::Housing => {
                let ratio = if game_state.available_housing_capacity > 0 { game_state.total_inhabitants as f32 / game_state.available_housing_capacity as f32 } else { 2.0 };
                let status = ratio < 1.0;
                (if status { "Adequate" } else { "Overcrowded" }, if status { Color::GREEN } else { Color::RED })
            },
            DiagnosticType::Healthcare => ("Nominal", Color::GREEN),
            DiagnosticType::Security => ("Nominal", Color::GREEN),
            DiagnosticType::Recreation => ("Nominal", Color::GREEN),
            DiagnosticType::Education => ("Nominal", Color::GREEN),
        };
        text.sections[0].value = format!("{:?}: {}", item.0, status_text);
        text.sections[0].style.color = color;
    }
}

fn update_research_panel_system(
    current_app: Res<CurrentApp>,
    game_state: Res<GameState>,
    mut list_panel_query: Query<Entity, With<AvailableResearchListPanel>>,
    mut commands: Commands,
){
    if current_app.0 != AppType::Research { return; }
    if !game_state.is_changed() && !current_app.is_changed() { return; }
    
    if let Ok(panel_entity) = list_panel_query.get_single_mut() {
        commands.entity(panel_entity).despawn_descendants();
        let all_techs = [Tech::BasicConstructionProtocols, Tech::EfficientExtraction];
        commands.entity(panel_entity).with_children(|parent| {
            for tech in all_techs {
                if !game_state.unlocked_techs.contains(&tech) {
                     parent.spawn((ButtonBundle{ style: Style { width: Val::Percent(100.0), padding: UiRect::all(Val::Px(8.0)), margin: UiRect::bottom(Val::Px(4.0)), ..default() }, background_color: NORMAL_BUTTON.into(), ..default()}, ResearchItemButton(tech)))
                     .with_children(|b| {b.spawn(TextBundle::from_section(format!("{:?}", tech), TextStyle { color: PRIMARY_TEXT_COLOR, ..default() }));});
                }
            }
        });
    }
}

fn research_item_button_system(
    mut selected_tech: ResMut<SelectedTech>,
    mut query: Query<(&Interaction, &ResearchItemButton, &mut BackgroundColor)>
){
     for (interaction, button, mut color) in query.iter_mut() {
        if selected_tech.0 == Some(button.0) { *color = ACTIVE_BUTTON.into(); } else { *color = NORMAL_BUTTON.into(); }
        if *interaction == Interaction::Pressed { 
            selected_tech.0 = if selected_tech.0 == Some(button.0) { None } else { Some(button.0) };
        }
         if *interaction == Interaction::Hovered && selected_tech.0 != Some(button.0) {
            *color = HOVERED_BUTTON.into();
        }
     }
}

fn update_research_details_panel_system(
    selected_tech: Res<SelectedTech>,
    game_state: Res<GameState>,
    mut panel_query: Query<Entity, With<ResearchDetailsPanel>>,
    mut commands: Commands,
){
    if !selected_tech.is_changed() { return; }

    if let Ok(panel) = panel_query.get_single_mut(){
        commands.entity(panel).despawn_descendants();
        if let Some(tech) = selected_tech.0 {
            commands.entity(panel).with_children(|parent|{
                parent.spawn(TextBundle::from_section(format!("{:?}", tech), TextStyle{font_size: 22.0, color: PRIMARY_TEXT_COLOR, ..default()}));
                let cost = game_state.tech_costs.get(&tech).unwrap_or(&0);
                parent.spawn(TextBundle::from_section(format!("Cost: {} Credits", cost), TextStyle{ color: LABEL_TEXT_COLOR, ..default()}));
                parent.spawn((ButtonBundle{style: Style{position_type: PositionType::Absolute, bottom: Val::Px(10.0), right: Val::Px(10.0), padding: UiRect::all(Val::Px(10.0)), ..default()}, background_color: NORMAL_BUTTON.into(), ..default()}, InitiateResearchButton))
                .with_children(|button| { button.spawn(TextBundle::from_section("RESEARCH", TextStyle{color: PRIMARY_TEXT_COLOR, ..default()})); });
            });
        }
    }
}

fn initiate_research_button_system(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<InitiateResearchButton>)>,
    selected_tech: Res<SelectedTech>,
    mut game_state: ResMut<GameState>
){
    if let Ok(Interaction::Pressed) = interaction_q.get_single() {
        if let Some(tech) = selected_tech.0 {
            let cost = *game_state.tech_costs.get(&tech).unwrap_or(&0) as f64;
            if game_state.credits >= cost {
                if game_state.research_progress.is_none() {
                    game_state.credits -= cost;
                    game_state.research_progress = Some((tech, 0.0));
                }
            }
        }
    }
}