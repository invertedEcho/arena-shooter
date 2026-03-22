use bevy::{color::palettes::css::GRAY, prelude::*};

#[derive(Component)]
struct BuyScreenRoot;

pub struct BuyScreenPlugin;

impl Plugin for BuyScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_buy_screen);
    }
}

fn spawn_buy_screen(mut commands: Commands) {
    commands.spawn((
        BuyScreenRoot,
        Node {
            width: percent(100),
            height: percent(100),
            margin: percent(5).into(),
            padding: percent(5).into(),
            ..default()
        },
        BackgroundColor(GRAY.with_alpha(0.6).into()),
    ));
}
