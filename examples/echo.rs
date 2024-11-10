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
        let mut buf = PacketBuf::new();
        buf
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

pub fn main() {
    let mut server: Server<ProtocolState, Packets> = Server::new();
    server.event_loop();
}