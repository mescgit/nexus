// src/main.rs

use bevy::prelude::*;
// REMOVED: `use bevy_prototype_lyon::prelude::*;`
use game_state::{
    BasicDwelling, BioDome, BuildingType, ColonyStats, Extractor, GameState, GraphData, PowerRelay,
    ResearchInstitute, ResourceType, SecurityStation, StorageSilo, Tech, WellnessPost,
};

mod game_state;

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
struct GraphArea; // Marker for the graph's background node

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum StatType {
    Housing,
    Jobs,
    Health,
    Police,
}

// --- Constants ---
const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

const HOUSING_COLOR: Color = Color::rgb(0.2, 0.7, 1.0);
const JOBS_COLOR: Color = Color::rgb(1.0, 0.7, 0.2);
const HEALTH_COLOR: Color = Color::rgb(0.2, 1.0, 0.7);
const POLICE_COLOR: Color = Color::rgb(1.0, 0.2, 0.2);

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
            // REMOVED: ShapePlugin
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
            .add_systems(Startup, setup_ui)
            .add_systems(Update, (
                button_interaction_system,
                research_button_system,
                update_text_display,
                draw_graph_gizmos, // REPLACED: draw_graph_system with a gizmo version
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
        // --- Top Bar ---
        parent.spawn(NodeBundle {
            style: Style { width: Val::Percent(100.0), padding: UiRect::all(Val::Px(10.0)), align_items: AlignItems::Center, flex_wrap: FlexWrap::Wrap, ..default() },
            background_color: Color::DARK_GRAY.into(), ..default()
        }).with_children(|parent| {
            parent.spawn((TextBundle::from_section("Power:", TextStyle { font_size: 20.0, ..default() }), PowerText));
            parent.spawn((TextBundle::from_section("Nutrient Paste:", TextStyle { font_size: 20.0, ..default() }).with_style(Style { margin: UiRect { left: Val::Px(20.0), ..default() }, ..default() }), ResourceText(ResourceType::NutrientPaste)));
            parent.spawn((TextBundle::from_section("Ferrocrete Ore:", TextStyle { font_size: 20.0, ..default() }).with_style(Style { margin: UiRect { left: Val::Px(20.0), ..default() }, ..default() }), ResourceText(ResourceType::FerrocreteOre)));
            parent.spawn((TextBundle::from_section("Housing:", TextStyle { font_size: 20.0, color: HOUSING_COLOR, ..default() }).with_style(Style { margin: UiRect { left: Val::Px(40.0), ..default() }, ..default() }), ColonyStatText(StatType::Housing)));
            parent.spawn((TextBundle::from_section("Jobs:", TextStyle { font_size: 20.0, color: JOBS_COLOR, ..default() }).with_style(Style { margin: UiRect { left: Val::Px(20.0), ..default() }, ..default() }), ColonyStatText(StatType::Jobs)));
            parent.spawn((TextBundle::from_section("Health:", TextStyle { font_size: 20.0, color: HEALTH_COLOR, ..default() }).with_style(Style { margin: UiRect { left: Val::Px(20.0), ..default() }, ..default() }), ColonyStatText(StatType::Health)));
            parent.spawn((TextBundle::from_section("Police:", TextStyle { font_size: 20.0, color: POLICE_COLOR, ..default() }).with_style(Style { margin: UiRect { left: Val::Px(20.0), ..default() }, ..default() }), ColonyStatText(StatType::Police)));
        });

        // --- Center Content (Graph Area) ---
        // This is now just a background panel. The gizmos will be drawn on top of it.
        parent.spawn((
            NodeBundle {
                style: Style { width: Val::Percent(100.0), height: Val::Percent(100.0), margin: UiRect::all(Val::Px(10.0)), ..default() },
                background_color: Color::rgba(0.1, 0.1, 0.1, 0.5).into(),
                ..default()
            },
            GraphArea, // Add marker to find its position and size
        ));

        // --- Bottom Bar ---
        parent.spawn(NodeBundle {
            style: Style { width: Val::Percent(100.0), padding: UiRect::all(Val::Px(5.0)), align_items: AlignItems::Center, justify_content: JustifyContent::SpaceBetween, ..default() },
            background_color: Color::DARK_GRAY.into(), ..default()
        }).with_children(|parent| {
            parent.spawn(NodeBundle { style: Style { flex_direction: FlexDirection::Row, flex_wrap: FlexWrap::Wrap, width: Val::Percent(75.0), ..default() }, ..default() })
                .with_children(|parent| {
                    let buildings = [
                        BuildingType::BioDome, BuildingType::Extractor, BuildingType::PowerRelay,
                        BuildingType::StorageSilo, BuildingType::ResearchInstitute,
                        BuildingType::BasicDwelling, BuildingType::WellnessPost, BuildingType::SecurityStation,
                    ];
                    for building_type in buildings {
                        parent.spawn((ButtonBundle { style: Style { padding: UiRect::all(Val::Px(8.)), margin: UiRect::all(Val::Px(3.)), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, background_color: NORMAL_BUTTON.into(), ..default() }, BuildButton(building_type)))
                                .with_children(|p| { p.spawn(TextBundle::from_section(format!("Build {:?}", building_type), TextStyle { font_size: 16.0, ..default() })); });
                    }
                });
            
            parent.spawn(NodeBundle { style: Style { align_items: AlignItems::Center, justify_content: JustifyContent::FlexEnd, flex_direction: FlexDirection::Row, ..default() }, ..default() })
                .with_children(|parent| {
                    parent.spawn((ButtonBundle { style: Style { padding: UiRect::all(Val::Px(8.)), margin: UiRect::horizontal(Val::Px(5.)), ..default() }, background_color: NORMAL_BUTTON.into(), ..default() }, ResearchButton(Tech::BasicConstructionProtocols)))
                            .with_children(|p| { p.spawn(TextBundle::from_section("Research Basic Construction", TextStyle { font_size: 16.0, ..default() })); });
                    parent.spawn((TextBundle::from_section("Welcome!", TextStyle { font_size: 20.0, ..default() }).with_style(Style{margin: UiRect::left(Val::Px(20.0)), ..default()}), MessageText));
                });
        });
    });
}

// NEW: This system uses Gizmos to draw the graph, avoiding UI conflicts.
fn draw_graph_gizmos(
    mut gizmos: Gizmos,
    graph_data: Res<GraphData>,
    graph_area_query: Query<(&Node, &GlobalTransform), With<GraphArea>>,
) {
    if graph_data.history.is_empty() { return; }

    let (graph_node, transform) = graph_area_query.single();
    let graph_area = graph_node.size();
    
    // The Gizmos are drawn in world space, so we need the bottom-left corner of our UI node.
    let bottom_left = transform.translation().truncate() - graph_area / 2.0;

    let max_val = graph_data.history.iter().fold(100.0f32, |max, stats| {
        max.max(stats.total_housing as f32).max(stats.total_jobs as f32).max(stats.health_capacity as f32).max(stats.police_capacity as f32)
    });

    let stat_types = [
        (StatType::Housing, HOUSING_COLOR),
        (StatType::Jobs, JOBS_COLOR),
        (StatType::Health, HEALTH_COLOR),
        (StatType::Police, POLICE_COLOR),
    ];

    for (stat_type, color) in stat_types {
        let mut points: Vec<Vec2> = Vec::new();
        for (i, stats) in graph_data.history.iter().enumerate() {
            let value = match stat_type {
                StatType::Housing => stats.total_housing as f32,
                StatType::Jobs => stats.total_jobs as f32,
                StatType::Health => stats.health_capacity as f32,
                StatType::Police => stats.police_capacity as f32,
            };
            
            let x = graph_area.x - (i as f32 * (graph_area.x / 200.0));
            let y = (value / max_val) * graph_area.y;
            
            if x >= 0.0 && y >= 0.0 {
                // Add the panel's bottom-left corner to translate to world space
                points.push(bottom_left + Vec2::new(x, y));
            }
        }

        if points.len() > 1 {
            gizmos.linestrip_2d(points, color);
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
                        BuildingType::BioDome => { commands.spawn(BioDome { power_required: 10, production_rate: 5.0 }); }
                        BuildingType::Extractor => { commands.spawn(Extractor { power_required: 15, resource_type: ResourceType::FerrocreteOre, extraction_rate: 2.5 }); }
                        BuildingType::PowerRelay => { commands.spawn(PowerRelay { power_output: 50 }); }
                        BuildingType::StorageSilo => { commands.spawn(StorageSilo { capacity: 1000 }); }
                        BuildingType::ResearchInstitute => { commands.spawn(ResearchInstitute); }
                        BuildingType::BasicDwelling => { commands.spawn(BasicDwelling { housing_capacity: 100 }); }
                        BuildingType::WellnessPost => { commands.spawn(WellnessPost { health_capacity: 50, jobs_provided: 5 }); }
                        BuildingType::SecurityStation => { commands.spawn(SecurityStation { police_capacity: 50, jobs_provided: 5 }); }
                    }
                } else {
                    log.message = "Not enough resources!".to_string();
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
                    log.message = format!("Researching {:?}...", tech);
                    game_state.research_progress = Some((tech, 0.0));
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
    )>,
    power_q: Query<&PowerRelay>,
    extractor_q: Query<&Extractor>,
    biodome_q: Query<&BioDome>,
) {
    for (mut text, resource_marker) in text_queries.p1().iter_mut() {
        let amount = game_state.current_resources.get(&resource_marker.0).unwrap_or(&0.0);
        text.sections[0].value = format!("{:?}: {:.1}", resource_marker.0, amount);
    }
    
    let power_gen: u32 = power_q.iter().map(|p| p.power_output).sum();
    let power_con: u32 = extractor_q.iter().map(|e| e.power_required).sum::<u32>() + biodome_q.iter().map(|b| b.power_required).sum::<u32>();
    for mut text in text_queries.p0().iter_mut() {
        text.sections[0].value = format!("Power: {} / {}", power_con, power_gen);
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
        for mut text in text_queries.p3().iter_mut() {
            text.sections[0].value = log.message.clone();
        }
    }
}