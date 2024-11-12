mod chat;

use std::net::{Ipv4Addr, SocketAddrV4};
use std::time::Duration;
use dragonet::server::Server;
use chat::{Packets, ProtocolState};

pub fn main() {
    let mut server: Server<ProtocolState, Packets> = Server::new();
    server_provider(&mut server);
    server.event_loop();
}

fn server_provider(server: &mut Server<ProtocolState, Packets>) -> &mut Server<ProtocolState, Packets> {
    server
        .with_address(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 2000))
        .with_startup_event(|server| {
            println!("Starting!");
        })
        .with_connection_event(|connection| {
            println!("Started connection");
            connection.set_state(ProtocolState::Chat);
            connection.send_packet(Packets::S2CChatMessage);
        })
        .with_packet_event(|connection, packet| {
            match packet {
                Packets::C2SChatMessage => {
                    println!("Received chat message on server!");
                    std::thread::spawn(move || {
                        std::thread::sleep(Duration::from_millis(1000));
                        connection.send_packet(Packets::S2CChatMessage);
                    }).join().unwrap();
                }
                _ => panic!("got clientbound packet somehow!")
            }
        })
}