use crate::buffer::Buffer;

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
    fn encode(&self) -> Buffer;
    fn decode(buf: &mut Buffer, meta: &PacketMetadata<S>) -> Self;
    fn metadata(&self) -> PacketMetadata<S>;
}
