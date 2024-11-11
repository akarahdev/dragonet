use std::sync::{Arc, Mutex};
use crate::protocol::{PacketState, Protocol};
use crate::server::conn::ServerConnection;

pub struct ConnectionRef<S, T>
where
    S: PacketState,
    T: Protocol<S>,
{
    pub(crate) connection: Arc<Mutex<ServerConnection<S, T>>>,
}

impl<S, T> ConnectionRef<S, T>
where
    S: PacketState,
    T: Protocol<S>,
{
    pub fn set_state(&self, state: S) {
        self.connection.lock().unwrap().state = Some(state);
    }

    pub fn send_packet(&self, packet: T) {
        self.connection.lock().unwrap().packet_queue.push(packet);
    }
}

impl<S, T> Clone for ConnectionRef<S, T>
where
    S: PacketState,
    T: Protocol<S>,
{
    fn clone(&self) -> Self {
        ConnectionRef { connection: self.connection.clone() }
    }
}

unsafe impl<S, T> Send for ConnectionRef<S, T>
where
    S: PacketState,
    T: Protocol<S>,
{}

unsafe impl<S, T> Sync for ConnectionRef<S, T>
where
    S: PacketState,
    T: Protocol<S>,
{}