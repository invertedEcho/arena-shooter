use bevy::prelude::*;

pub struct GameScorePlugin;

impl Plugin for GameScorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameScore::default());
    }
}

#[derive(Resource)]
pub struct GameScore {
    pub player: u64,
    pub enemy: u64,
}

impl Default for GameScore {
    fn default() -> Self {
        GameScore {
            player: 0,
            enemy: 0,
        }
    }
}
