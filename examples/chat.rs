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
    C2SChatMessage(String),
    S2CChatMessage(String),
}

impl Protocol<ProtocolState> for Packets {
    fn encode(&self) -> Buffer {
        let mut buf = Buffer::new();
        match self {
            Packets::C2SChatMessage(str) => {
                buf.write_string(str);
            }
            Packets::S2CChatMessage(str) => {
                buf.write_string(str);
            }
        };
        buf
    }

    fn decode(buf: &mut Buffer, meta: &PacketMetadata<ProtocolState>) -> Self {
        match meta.direction {
            PacketDirection::Clientbound => match meta.state {
                ProtocolState::Chat => match meta.id {
                    0x00 => Packets::S2CChatMessage(buf.read_string()),
                    _ => panic!("unknown packet"),
                }
            }
            PacketDirection::Serverbound =>  match meta.state {
                ProtocolState::Chat => match meta.id {
                    0x00 => Packets::C2SChatMessage(buf.read_string()),
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
            Packets::C2SChatMessage(msg) => PacketMetadata {
                id: 0,
                state: ProtocolState::Chat,
                direction: PacketDirection::Serverbound,
            },
            Packets::S2CChatMessage(msg) => PacketMetadata {
                id: 0,
                state: ProtocolState::Chat,
                direction: PacketDirection::Clientbound,
            },
        }
    }
}
