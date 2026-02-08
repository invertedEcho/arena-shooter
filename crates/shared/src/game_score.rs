use bevy::{platform::collections::HashMap, prelude::*};
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, PartialEq, Clone, Reflect)]
pub struct GameScore {
    /// key is client id/peer id
    pub players: HashMap<u64, PlayerStats>,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Reflect, Default)]
pub struct PlayerStats {
    pub username: String,
    pub kills: u64,
    pub deaths: u64,
}

pub struct GameScoreDelta {
    updated_players: HashMap<u64, PlayerStats>,
    removed_players: Vec<u64>,
}

impl Diffable<GameScoreDelta> for GameScore {
    fn base_value() -> Self {
        Self {
            players: HashMap::new(),
        }
    }

    fn diff(&self, new: &Self) -> GameScoreDelta {
        let mut updated_players = HashMap::new();
        let mut removed_players = Vec::new();

        for (client_id, new_score) in &new.players {
            match self.players.get(client_id) {
                // no change to score
                Some(old_score) if old_score == new_score => {}
                _ => {
                    // either new player or score was updated
                    updated_players.insert(*client_id, new_score.clone());
                }
            }
        }

        for client_id in self.players.keys() {
            if !new.players.contains_key(client_id) {
                removed_players.push(*client_id);
            }
        }

        GameScoreDelta {
            updated_players,
            removed_players,
        }
    }

    fn apply_diff(&mut self, delta: &GameScoreDelta) {
        for (client_id, new_score) in &delta.updated_players {
            self.players.insert(*client_id, new_score.clone());
        }

        for client_id in &delta.removed_players {
            self.players.remove(client_id);
        }
    }
}

pub fn get_random_unused_client_id(game_score: &GameScore) -> u64 {
    for i in 0..1000 {
        if game_score.players.keys().find(|id| **id == i).is_none() {
            return i;
        }
    }
    0
}
