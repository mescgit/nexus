use bevy::prelude::*;

use crate::game_state::{GameState, Tech};

pub fn research_system(mut game_state: ResMut<GameState>) {
    if game_state.research_institutes.iter().all(|ri| !ri.is_staffed) {
        return;
    }
    let mut completed_tech: Option<Tech> = None;
    if let Some((tech, progress)) = &game_state.research_progress {
        let required_progress = game_state.tech_costs[tech] as f32;
        if progress + 1.0 >= required_progress {
            completed_tech = Some(*tech);
        }
    }
    if let Some(tech) = completed_tech {
        game_state.unlocked_techs.insert(tech);
        game_state.research_progress = None;
    } else if let Some((_, progress)) = &mut game_state.research_progress {
        *progress += 1.0;
    }
}