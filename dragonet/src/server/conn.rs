use std::marker::PhantomData;
use std::net::TcpStream;
use crate::protocol::{PacketState, Protocol};

pub struct ServerConnection<S, T>
where
    S: PacketState,
    T: Protocol<S>,
{
    pub(crate) stream: TcpStream,
    pub(crate) packet_queue: Vec<T>,
    pub(crate) state: Option<S>,
    pub(crate) _phantom: PhantomData<(S, T)>,
}

impl<S, T> ServerConnection<S, T>
where
    S: PacketState,
    T: Protocol<S>,
{
    pub fn set_state(&mut self, state: S) -> &mut ServerConnection<S, T> {
        self.state = Some(state);
        self
    }

    pub fn send_packet(&mut self, packet: T) -> &mut ServerConnection<S, T> {
        // TODO: write to connection
        self.packet_queue.push(packet);
        self
    }
}