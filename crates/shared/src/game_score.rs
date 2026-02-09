use bevy::{platform::collections::HashMap, prelude::*};
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Component, Serialize, Deserialize, PartialEq, Clone, Reflect, Debug,
)]
pub struct GameScore {
    // for players, we use the PeerId converted into bits as entities will differ on client and server
    pub players: HashMap<u64, LivingEntityStats>,
    // as enemies only exists in singleplayer, we can just use Entity
    pub enemies: HashMap<Entity, LivingEntityStats>,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Reflect, Default, Debug)]
pub struct LivingEntityStats {
    pub username: String,
    pub kills: u64,
    pub deaths: u64,
}

pub struct GameScoreDelta {
    updated_players: HashMap<u64, LivingEntityStats>,
    removed_players: Vec<u64>,
    updated_enemies: HashMap<Entity, LivingEntityStats>,
    removed_enemies: Vec<Entity>,
}

impl Diffable<GameScoreDelta> for GameScore {
    fn base_value() -> Self {
        Self {
            players: HashMap::new(),
            enemies: HashMap::new(),
        }
    }

    fn diff(&self, new: &Self) -> GameScoreDelta {
        let mut updated_players = HashMap::new();
        let mut removed_players = Vec::new();

        let mut updated_enemies = HashMap::new();
        let mut removed_enemies = Vec::new();

        for (entity, new_score) in &new.players {
            match self.players.get(entity) {
                // no change to score
                Some(old_score) if old_score == new_score => {}
                _ => {
                    // either new player or score was updated
                    updated_players.insert(*entity, new_score.clone());
                }
            }
        }

        for peer_id in self.players.keys() {
            if !new.players.contains_key(peer_id) {
                removed_players.push(*peer_id);
            }
        }

        for (entity, new_score) in &new.enemies {
            match self.enemies.get(entity) {
                // no change to score
                Some(old_score) if old_score == new_score => {}
                _ => {
                    // either new enemy or score was updated
                    updated_enemies.insert(*entity, new_score.clone());
                }
            }
        }

        for entity in self.enemies.keys() {
            if !new.enemies.contains_key(entity) {
                removed_enemies.push(*entity);
            }
        }

        GameScoreDelta {
            updated_players,
            removed_players,
            updated_enemies,
            removed_enemies,
        }
    }

    fn apply_diff(&mut self, delta: &GameScoreDelta) {
        for (peer_id, new_score) in &delta.updated_players {
            self.players.insert(*peer_id, new_score.clone());
        }

        for peer_id in &delta.removed_players {
            self.players.remove(peer_id);
        }

        for (entity, new_score) in &delta.updated_enemies {
            self.enemies.insert(*entity, new_score.clone());
        }

        for entity in &delta.removed_enemies {
            self.enemies.remove(entity);
        }
    }
}
