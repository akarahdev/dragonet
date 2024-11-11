use crate::buffer::PacketBuf;

#[derive(Clone, Copy)]
pub enum PacketDirection {
    Clientbound,
    Serverbound
}

pub trait PacketState: Clone {
    fn get_state_by_id(id: u8) -> Self; 
}

#[derive(Clone, Copy)]
pub struct PacketMetadata<S: PacketState> {
    pub id: u32,
    pub state: S,
    pub direction: PacketDirection
}

pub trait Protocol<S: PacketState> {
    fn encode(&self) -> PacketBuf;
    fn decode(buf: &mut PacketBuf, meta: &PacketMetadata<S>) -> Self;
    fn size_of(&self) -> u32;
    fn metadata(&self) -> PacketMetadata<S>;
}
