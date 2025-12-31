use bevy::prelude::*;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use shared::SERVER_ADDRESS;

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(ServerPlugins::default());
    app.add_systems(Startup, setup_server);
    app.add_observer(handle_new_client);
    app.run();
}

pub fn setup_server(mut commands: Commands) {
    let server = commands
        .spawn((
            NetcodeServer::new(NetcodeConfig::default()),
            LocalAddr(SERVER_ADDRESS),
            ServerUdpIo::default(),
        ))
        .id();

    commands.trigger(Start { entity: server });
}

pub fn handle_new_client(trigger: On<Add, Connected>) {
    info!("New client! {}", trigger.entity);
}
