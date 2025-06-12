use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Serialize, Deserialize, Clone)]
pub struct PopulationResource {
    pub count: u32,
}

impl Default for PopulationResource {
    fn default() -> Self {
        PopulationResource { count: 5 }
    }
}
