use std::env;

pub const LOCAL_SERVER_PRIVATE_KEY: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0,
];

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
