use bevy::prelude::*;
use netvy::prelude::*;
use shared::{enemy::components::Enemy, player::Player};

pub type OurPlayerFilter = (With<Player>, With<Owned>);

pub type PlayerOrEnemyFilter = Or<(With<Enemy>, With<Player>)>;
