use bevy::prelude::*;
use crate::game_state::GameState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticType {
    NutrientPaste,
    Housing,
    Healthcare,
    Security,
    Recreation,
    Education,
}
use super::*;

#[derive(Component)]
pub(super) struct ColonyStatusPanel;
#[derive(Component)]
pub(super) struct DiagnosticItem(pub DiagnosticType);

pub(super) fn build(viewport: &mut ChildBuilder, _assets: &Res<AssetServer>) {

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
}
pub(super) fn update_colony_status_panel_system(
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
  
