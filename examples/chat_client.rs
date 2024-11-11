mod chat;

use std::io::stdin;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::{Arc, Mutex};
use dragonet::server::Server;
use chat::{Packets, ProtocolState};
use dragonet::client::Client;
use crate::chat::Packets::C2SChatMessage;

pub fn main() {
    let mut client: Client<ProtocolState, Packets> = Client::new();
    client_provider(&mut client);
    client.event_loop();
}

fn client_provider(client: &mut Client<ProtocolState, Packets>) -> &mut Client<ProtocolState, Packets> {
    client
        .with_address(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 2000))
        .on_connect(|clr| {
            std::thread::spawn(move || {
                clr.set_state(ProtocolState::Chat);
                clr.send_packet(Packets::S2CChatMessage);
            });
        })
        .with_packet_event(|connection, packet| {
            match packet {
                Packets::S2CChatMessage => {
                    println!("Received a message!");
                    connection.send_packet(Packets::C2SChatMessage)
                }
                p => panic!("got serverbound packet somehow! {:?}", p)
            }
        })
}