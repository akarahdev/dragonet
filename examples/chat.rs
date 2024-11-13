use dragonet::buffer::Buffer;
use dragonet::protocol::{PacketDirection, PacketMetadata, PacketState, Protocol};

#[derive(Clone, Copy)]
pub enum ProtocolState {
    Chat,
}

impl PacketState for ProtocolState {
    fn get_state_by_id(id: u8) -> Self {
        match id {
            0 => ProtocolState::Chat,
            _ => panic!("unknown state")
        }
    }
}

#[derive(Debug)]
pub enum Packets {
    C2SChatMessage,
    S2CChatMessage,
}

impl Protocol<ProtocolState> for Packets {
    fn encode(&self) -> Buffer {
        Buffer::new()
    }

    fn decode(buf: &mut Buffer, meta: &PacketMetadata<ProtocolState>) -> Self {
        match meta.direction {
            PacketDirection::Clientbound => match meta.state {
                ProtocolState::Chat => match meta.id {
                    0x00 => Packets::S2CChatMessage,
                    _ => panic!("unknown packet"),
                }
            }
            PacketDirection::Serverbound =>  match meta.state {
                ProtocolState::Chat => match meta.id {
                    0x00 => Packets::C2SChatMessage,
                    _ => panic!("unknown packet"),
                }
            }
        }
    }

    fn size_of(&self) -> u32 {
        1
    }

    fn metadata(&self) -> PacketMetadata<ProtocolState> {
        match self {
            Packets::C2SChatMessage => PacketMetadata {
                id: 0,
                state: ProtocolState::Chat,
                direction: PacketDirection::Serverbound,
            },
            Packets::S2CChatMessage => PacketMetadata {
                id: 0,
                state: ProtocolState::Chat,
                direction: PacketDirection::Clientbound,
            },
        }
    }
}
