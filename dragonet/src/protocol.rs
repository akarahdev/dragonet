use crate::buffer::PacketBuf;

pub trait Protocol {
    fn encode(&self, packet: &Self) -> PacketBuf;
    fn decode(&self, buf: PacketBuf) -> Self;
    fn size_of(packet: &Self) -> u32;
}