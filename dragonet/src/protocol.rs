use crate::buffer::PacketBuf;

pub enum PacketDirection {
    Clientbound,
    Serverbound
}

pub trait PacketState {
    fn get_state_by_id(id: u8) -> Self; 
}

pub struct PacketMetadata<S: PacketState> {
    id: u32,
    state: S,
    direction: PacketDirection
}

pub trait Protocol<S: PacketState> {
    fn encode(&self) -> PacketBuf;
    fn decode(&self, buf: PacketBuf) -> Self;
    fn size_of(&self) -> u32;
    fn metadata(&self) -> PacketMetadata<S>;
}
