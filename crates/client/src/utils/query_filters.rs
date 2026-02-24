use bevy::prelude::*;
use lightyear::prelude::*;
use shared::{enemy::components::Enemy, player::Player};

pub type OurPlayerFilter = (With<Player>, With<Controlled>);

pub type PlayerOrEnemyFilter = Or<(With<Enemy>, With<Player>)>;
