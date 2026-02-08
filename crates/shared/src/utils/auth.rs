use std::env;

use crate::ServerMode;

pub fn load_private_key_from_env() -> Result<[u8; 32], String> {
    let Ok(value) = env::var("SERVER_PRIVATE_KEY") else {
        panic!("Please create a .env file containing a SERVER_PRIVATE_KEY.");
    };

    let bytes = hex::decode(&value)
        .map_err(|e| format!("Invalid hex in SERVER_PRIVATE_KEY: {e}"))?;

    if bytes.len() != 32 {
        return Err(format!(
            "SERVER_PRIVATE_KEY must be 32 bytes (got {})",
            bytes.len()
        ));
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Ok(key)
}

// Dependin whether this is for server binary or server locally for single player, it will either:
// - return private key from .env file (server binary)
// - return static private key, just zeroes (local server for singleplayer on the client)
pub fn get_private_key(server_mode: &ServerMode) -> [u8; 32] {
    const LOCAL_SERVER_PRIVATE_KEY: [u8; 32] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
    ];
    match server_mode {
        ServerMode::RemoteServer => load_private_key_from_env().unwrap(),
        ServerMode::LocalServerSinglePlayer => LOCAL_SERVER_PRIVATE_KEY,
        ServerMode::None => LOCAL_SERVER_PRIVATE_KEY,
    }
}
