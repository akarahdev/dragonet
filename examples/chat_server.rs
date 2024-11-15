use std::net::{Ipv4Addr, SocketAddrV4};
use dragonet::server::Server;
use dragonet_macros::server;
use crate::chat_protocol::{Packets, ProtocolState};

pub mod chat_protocol;

#[server]
pub fn server_provider(server: &mut Server<ProtocolState, Packets>) -> &mut Server<ProtocolState, Packets> {
    server_provider_impl(server)
}

pub fn server_provider_impl(server: &mut Server<ProtocolState, Packets>) -> &mut Server<ProtocolState, Packets> {
    server
        .with_address(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 2000))
        .with_connection_event(|conn| {
            println!("Connected!");
            conn.set_state(ProtocolState::Chat);
            conn.send_packet(Packets::ClientboundChatMessage("You connected! Hi!".to_string()))
        })
        .with_packet_event(|conn, packet| {
            conn.send_packet(Packets::ClientboundChatMessage("Recv".to_string()));
        })
}