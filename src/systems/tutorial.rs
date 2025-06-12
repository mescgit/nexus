use bevy::prelude::*;

use crate::resources::tutorial::{TutorialState, TooltipStep};

pub fn tutorial_system(state: Res<TutorialState>) {
    let _steps: Vec<TooltipStep> = crate::resources::tutorial::get_tutorial_steps();
    let _ = state.current_step; // placeholder to avoid unused warnings
}
