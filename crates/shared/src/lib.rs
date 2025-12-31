use std::net::{IpAddr, Ipv4Addr, SocketAddr};

mod protocol;

const SERVER_PORT: u16 = 5888;
pub const SERVER_ADDRESS: SocketAddr =
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), SERVER_PORT);
