use std::net::{Ipv4Addr, SocketAddrV4};
use dragonet::server::Server;
use dragonet_macros::server;
use crate::chat_protocol::{Packets, ProtocolState};

pub mod chat_protocol;

#[server]
pub fn server_provider(server: &mut Server<ProtocolState, Packets>) -> &mut Server<ProtocolState, Packets> {
    server
        .with_address(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 2000))
}