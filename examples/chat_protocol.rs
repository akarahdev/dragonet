use dragonet::buffer::Buffer;
use dragonet::protocol::{PacketDirection, PacketMetadata, PacketState, Protocol};
use crate::chat_protocol::Packets::{ClientboundChatMessage, ServerboundChatMessage};
use crate::chat_protocol::ProtocolState::Chat;

#[derive(Clone)]
pub enum ProtocolState {
    Chat
}

impl PacketState for ProtocolState {
    fn get_state_by_id(id: u8) -> Self {
        match id {
            0 => Chat,
            _ => Chat
        }
    }
}

#[derive(Debug)]
pub enum Packets {
    ServerboundChatMessage(String),
    ClientboundChatMessage(String)
}

impl Protocol<ProtocolState> for Packets {
    fn encode(&self) -> Buffer {
        let mut buf = Buffer::new();
        match self {
            &ServerboundChatMessage(ref content) => {
                buf.write_var_int(0);
                buf.write_string(content);
            }
            &ClientboundChatMessage(ref content) => {
                buf.write_var_int(0);
                buf.write_string(content);
            }
        }
        buf
    }

    fn decode(buf: &mut Buffer, meta: &PacketMetadata<ProtocolState>) -> Self {
        match meta.direction {
            PacketDirection::Serverbound =>
                match meta.id {
                    0 => ServerboundChatMessage(buf.read_string()),
                    _ => panic!("unknown packet")
                }
            PacketDirection::Clientbound =>
                match meta.id {
                    0 => ClientboundChatMessage(buf.read_string()),
                    _ => panic!("unknown packet")
                }
        }
    }

    fn metadata(&self) -> PacketMetadata<ProtocolState> {
        match self {
            ServerboundChatMessage(content) => PacketMetadata {
                id: 0,
                state: Chat,
                direction: PacketDirection::Serverbound,
            },
            ClientboundChatMessage(content) => PacketMetadata {
                id: 0,
                state: Chat,
                direction: PacketDirection::Clientbound,
            }
        }
    }
}