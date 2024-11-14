use dragonet::protocol::{PacketDirection, PacketMetadata, Protocol};
use crate::chat_protocol::Packets;
use crate::chat_protocol::ProtocolState::Chat;

pub mod chat_protocol;

fn main() {
    let packet = Packets::ServerboundChatMessage("Hello world!".to_string());
    let mut encoded = packet.encode();
    let packet_id = encoded.read_var_int();
    let decoded = Packets::decode(&mut encoded, &PacketMetadata {
        id: packet_id as u32,
        state: Chat,
        direction: PacketDirection::Serverbound,
    });
    println!("{:?}", packet);
    println!("{:?}", encoded);
    println!("{:?}", decoded);
}