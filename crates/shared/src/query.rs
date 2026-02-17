use bevy::prelude::*;
use lightyear::prelude::*;

use crate::player::Player;

pub type OurPlayerQueryFilter = (With<Player>, With<Controlled>);
