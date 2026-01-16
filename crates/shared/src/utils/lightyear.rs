pub enum DisconnectReason {
    ClientTriggered,
    Unknown,
}

pub fn parse_lightyear_disconnect_reason(
    disconnect_reason: &str,
) -> DisconnectReason {
    if disconnect_reason.contains("Client trigger") {
        return DisconnectReason::ClientTriggered;
    }
    DisconnectReason::Unknown
}
