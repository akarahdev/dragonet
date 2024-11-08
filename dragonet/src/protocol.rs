pub trait Protocol {
    fn encode(&self, packet: &Self) -> Vec<u8>;
    fn decode(&self, bytes: Vec<u8>) -> Self;
    fn size_of(packet: &Self) -> u32;
}