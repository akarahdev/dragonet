mod chat;

use std::net::{Ipv4Addr, SocketAddrV4};
use std::time::Duration;
use dragonet::server::Server;
use chat::{Packets, ProtocolState};
use dragonet_macros::server;

#[server]
fn server_provider(server: &mut Server<ProtocolState, Packets>) -> &mut Server<ProtocolState, Packets> {
    server
        .with_address(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 2000))
        .with_startup_event(|server| {
            println!("Starting!");
        })
        .with_connection_event(|connection| {
            println!("Started connection");
            connection.set_state(ProtocolState::Chat);
        })
        .with_packet_event(|connection, packet| {
            match packet {
                Packets::C2SChatMessage(msg) => {
                    println!("Message: {}", msg)
                }
                _ => panic!("got clientbound packet somehow!")
            }
        })
}