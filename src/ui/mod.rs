// src/ui/mod.rs

use bevy::prelude::*;
use crate::game_state::{
    GameState,
    GraphData,
    LoadGameEvent,
    ResourceType,
    SaveGameEvent,
    Tech,
    ALL_BUILDING_TYPES,
};
use crate::game_state::{BuildingType as GameBuildingType, DevelopmentPhase, ServiceType, ZoneType};
mod dashboard;
mod construction;
mod colony_status;
mod research;
use dashboard::{DashboardPanel, ManagedStructuresPanel, ZoneListButton, ZoneDetailsPanel, UpgradeZoneButton, RemoveZoneButton, AssignSpecialistToZoneButton, UnassignSpecialistFromZoneButton};
use construction::ConstructionPanel;
use colony_status::ColonyStatusPanel;
use research::ResearchPanel;

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
#[derive(Component)]
struct AppDrawerButton(AppType);
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

#[derive(Component)]
pub struct UiTag(pub &'static str);

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



pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentApp>()
            .init_resource::<CurrentConstructionCategory>()
            .init_resource::<SelectedBuilding>()
            .init_resource::<SelectedTech>()
            .init_resource::<SelectedZone>()
            .add_systems(Startup, setup_ui)
                .add_systems(Update, (
                    app_drawer_button_system,
                    manage_app_panels_visibility,
                    update_status_ticker_system,
                    dashboard::update_dashboard_notifications_system,
                    dashboard::update_admin_spire_panel_system,
                    dashboard::update_legacy_structure_panel_system,
                    dashboard::update_managed_structures_panel_system,
                    dashboard::zone_list_button_interaction_system,
                    dashboard::upgrade_zone_button_interaction_system,
                    dashboard::remove_zone_button_interaction_system,
                    dashboard::assign_specialist_to_zone_button_interaction_system,
                    dashboard::unassign_specialist_from_zone_button_interaction_system,
                    dashboard::admin_spire_button_interaction_system,
                    dashboard::legacy_structure_button_system,
                    dashboard::draw_graph_gizmos,
                    construction::construction_category_tab_system,
                    dashboard::save_load_button_system,
                ))
                .add_systems(Update, (
                    construction::update_construction_list_system,
                    construction::construction_item_interaction_system,
                    construction::update_construction_details_panel_system,
                    construction::construction_interaction_system,
                    construction::habitation_construction_system,
                    construction::service_construction_system,
                    construction::zone_construction_system,
                    colony_status::update_colony_status_panel_system,
                    research::update_research_panel_system,
                    research::research_item_button_system,
                    research::update_research_details_panel_system,
                    research::initiate_research_button_system,
                ));
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
        parent.spawn((NodeBundle {
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
        }, UiTag("ui.resources_panel"))).with_children(|ticker| {
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

                dashboard::build(viewport, &asset_server);
                construction::build(viewport, &asset_server);
                colony_status::build(viewport, &asset_server);
                research::build(viewport, &asset_server);
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
