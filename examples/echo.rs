use std::net::{Ipv4Addr, SocketAddrV4};
use dragonet::buffer::PacketBuf;
use dragonet::server::Server;
use dragonet::protocol::{PacketDirection, PacketMetadata, PacketState, Protocol};
enum ProtocolState {
    Handshaking
}

impl PacketState for ProtocolState {
    fn get_state_by_id(id: u8) -> Self {
        match id {
            0 => ProtocolState::Handshaking,
            _ => ProtocolState::Handshaking
        }
    }
}

enum Packets {
    HandshakingHandshake
}

impl Protocol<ProtocolState> for Packets {
    fn encode(&self) -> PacketBuf {
        
        PacketBuf::new()
    }

    fn decode(&self, buf: PacketBuf) -> Self {
        Packets::HandshakingHandshake
    }

    fn size_of(&self) -> u32 {
        0
    }

    fn metadata(&self) -> PacketMetadata<ProtocolState> {
        PacketMetadata {
            id: 0,
            state: ProtocolState::Handshaking,
            direction: PacketDirection::Serverbound,
        }
    }
}

// TODO: turn into proc macro
pub fn main() {
    let mut server: Server<ProtocolState, Packets> = Server::new();
    server_provider(&mut server);
    server.event_loop();
}

fn server_provider(server: &mut Server<ProtocolState, Packets>) -> &mut Server<ProtocolState, Packets> {
    server
        .with_address(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 2000))
        .with_packet_event(|connection, packet| {
            
        })

}