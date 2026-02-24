use bevy::prelude::*;
use lightyear::prelude::*;
use shared::player::Player;

pub type OurPlayerFilter = (With<Player>, With<Controlled>);
