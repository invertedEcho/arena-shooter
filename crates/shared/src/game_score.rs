use bevy::{platform::collections::HashMap, prelude::*};
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, PartialEq, Clone, Reflect)]
pub struct GameScore {
    /// key is the given entity (player or enemy) on the server
    pub living_entities: HashMap<Entity, LivingEntityStats>,
}

/// This message is only sent once, when the connection between client and server is established
/// The message gets sent from server to client, any further updates will be done via
/// Diffable<GameScore> singleton entity.
#[derive(Serialize, Deserialize)]
pub struct InitialGameScoreSyncMessage(pub GameScore);

#[derive(Serialize, Deserialize, PartialEq, Clone, Reflect, Default, Debug)]
pub struct LivingEntityStats {
    pub username: String,
    pub kills: u64,
    pub deaths: u64,
}

pub struct GameScoreDelta {
    updated_living_entities: HashMap<Entity, LivingEntityStats>,
    removed_living_entities: Vec<Entity>,
}

impl Diffable<GameScoreDelta> for GameScore {
    // FIXME: i think this is reason why in real multiplayer, per default it will be empty. so we
    // need a way to have initial sync for new entities. i guess we can just send a message from
    // server to client on initial connect
    fn base_value() -> Self {
        Self {
            living_entities: HashMap::new(),
        }
    }

    fn diff(&self, new: &Self) -> GameScoreDelta {
        let mut updated_players = HashMap::new();
        let mut removed_players = Vec::new();

        for (entity, new_score) in &new.living_entities {
            match self.living_entities.get(entity) {
                // no change to score
                Some(old_score) if old_score == new_score => {}
                _ => {
                    // either new player or score was updated
                    updated_players.insert(*entity, new_score.clone());
                }
            }
        }

        for entity in self.living_entities.keys() {
            if !new.living_entities.contains_key(entity) {
                removed_players.push(*entity);
            }
        }

        GameScoreDelta {
            updated_living_entities: updated_players,
            removed_living_entities: removed_players,
        }
    }

    fn apply_diff(&mut self, delta: &GameScoreDelta) {
        for (entity, new_score) in &delta.updated_living_entities {
            self.living_entities.insert(*entity, new_score.clone());
        }

        for entity in &delta.removed_living_entities {
            self.living_entities.remove(entity);
        }
    }
}
