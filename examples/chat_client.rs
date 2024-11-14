use std::net::{Ipv4Addr, SocketAddrV4};
use dragonet::client::Client;
use dragonet::protocol::PacketState;
use dragonet_macros::client;
use crate::chat_protocol::{Packets, ProtocolState};

pub mod chat_protocol;

#[client]
pub fn client_provider(client: &mut Client<ProtocolState, Packets>) -> &mut Client<ProtocolState, Packets> {
    client
        .with_address(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 2000))
}