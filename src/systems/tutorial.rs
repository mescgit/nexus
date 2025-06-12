use bevy::prelude::*;

use crate::resources::tutorial::TutorialState;

pub struct TutorialPlugin;

impl Plugin for TutorialPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TutorialState>()
            .add_systems(Update, tutorial_system);
    }
}

pub fn tutorial_system(world: &mut World) {
    let (trigger, title, content, action, step_index, total_steps) = {
        let state = world.resource::<TutorialState>();
        if state.current_step >= state.steps.len() {
            return;
        }
        let step = &state.steps[state.current_step];
        (
            step.trigger,
            step.title,
            step.content,
            step.required_action,
            state.current_step,
            state.steps.len(),
        )
    };

    if trigger(world) {
        info!(
            "[Tutorial {}/{}] {} - {}",
            step_index + 1,
            total_steps,
            title,
            content
        );
        if let Some(action_fn) = action {
            action_fn(world);
        }
        world.resource_mut::<TutorialState>().current_step += 1;
    }
}
