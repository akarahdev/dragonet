mod chat;

use std::net::{Ipv4Addr, SocketAddrV4};
use std::time::Duration;
use chat::{Packets, ProtocolState};
use dragonet::client::Client;

pub fn main() {
    let mut client: Client<ProtocolState, Packets> = Client::new();
    client_provider(&mut client);
    client.event_loop();
}

fn client_provider(client: &mut Client<ProtocolState, Packets>) -> &mut Client<ProtocolState, Packets> {
    client
        .with_address(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 2000))
        .on_connect(|clr| {
            println!("Started connection");
            clr.set_state(ProtocolState::Chat);
            clr.send_packet(Packets::C2SChatMessage);
        })
        .with_packet_event(|connection, packet| {
            match packet {
                Packets::S2CChatMessage => {
                    println!("Received a message!");
                    std::thread::spawn(move || {
                        std::thread::sleep(Duration::from_millis(1000));
                        connection.send_packet(Packets::C2SChatMessage);
                    }).join().unwrap();
                }
                p => panic!("got serverbound packet somehow! {:?}", p)
            }
        })
}