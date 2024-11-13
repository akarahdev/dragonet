mod chat;

use std::net::{Ipv4Addr, SocketAddrV4};
use std::time::Duration;
use chat::{Packets, ProtocolState};
use dragonet::client::Client;
use dragonet_macros::client;
use crate::chat::Packets::C2SChatMessage;

#[dragonet_macros::client]
fn client_provider(client: &mut Client<ProtocolState, Packets>) -> &mut Client<ProtocolState, Packets> {
    client
        .with_address(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 2000))
        .on_connect(|clr| {
            println!("Started connection");
            clr.set_state(ProtocolState::Chat);

            std::thread::spawn(move || {
                loop {
                    let mut line = String::new();
                    std::io::stdin().read_line(&mut line).unwrap();
                    clr.send_packet(C2SChatMessage(line));
                }
            });
        })
        .with_packet_event(|connection, packet| {
            match packet {
                Packets::S2CChatMessage(msg) => {
                    println!("Received a message! {}", msg);
                }
                p => panic!("got serverbound packet somehow! {:?}", p)
            }
        })
}