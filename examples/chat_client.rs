use std::io::stdin;
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
        .on_connect(|crf| {
            crf.send_packet(Packets::ServerboundChatMessage("I connected!".to_string()));
            std::thread::spawn(move || {
                crf.set_state(ProtocolState::Chat);
                loop {
                    let mut line = String::new();
                    stdin().read_line(&mut line).unwrap();
                    println!("{}", line);
                    crf.send_packet(Packets::ServerboundChatMessage(line));
                }
            });
        })
        .with_packet_event(|crf, packet| {
            match packet {
                Packets::ClientboundChatMessage(message) => {
                    println!("> {}", message)
                }
                _ => panic!("got serverboudn packet {:?}", packet)
            }
        })
}