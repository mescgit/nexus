use bevy::prelude::*;
use crate::game_state::{self, GameState, Tech};
use super::*;

#[derive(Component)]
pub(super) struct ResearchPanel;
#[derive(Component)]
pub(super) struct AvailableResearchListPanel;
#[derive(Component)]
pub(super) struct ResearchItemButton(pub Tech);
#[derive(Component)]
pub(super) struct ResearchDetailsPanel;
#[derive(Component)]
pub(super) struct InitiateResearchButton;

pub(super) fn build(viewport: &mut ChildBuilder, _assets: &Res<AssetServer>) {
    viewport
        .spawn((
            NodeBundle {
                style: Style {
                    display: Display::None,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
            ResearchPanel,
        ))
        .with_children(|research| {
            research.spawn(
                TextBundle::from_section(
                    "RESEARCH & DEVELOPMENT",
                    TextStyle {
                        font_size: 28.0,
                        color: BORDER_COLOR,
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                }),
            );

            research
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        flex_grow: 1.0,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|main| {
                    main.spawn((
                        NodeBundle {
                            style: Style {
                                width: Val::Percent(40.0),
                                height: Val::Percent(100.0),
                                border: UiRect::all(Val::Px(1.0)),
                                padding: UiRect::all(Val::Px(5.0)),
                                flex_direction: FlexDirection::Column,
                                ..default()
                            },
                            border_color: BORDER_COLOR.into(),
                            ..default()
                        },
                        AvailableResearchListPanel,
                    ));

                    main.spawn((
                        NodeBundle {
                            style: Style {
                                flex_grow: 1.0,
                                height: Val::Percent(100.0),
                                border: UiRect::all(Val::Px(1.0)),
                                padding: UiRect::all(Val::Px(10.0)),
                                margin: UiRect::left(Val::Px(10.0)),
                                flex_direction: FlexDirection::Column,
                                ..default()
                            },
                            border_color: BORDER_COLOR.into(),
                            ..default()
                        },
                        ResearchDetailsPanel,
                    ));
                });
        });
}
pub(super) fn update_research_panel_system(
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

pub(super) fn research_item_button_system(
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

pub(super) fn update_research_details_panel_system(
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

pub(super) fn initiate_research_button_system(
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

// --- New Systems for Legacy Structure ---

