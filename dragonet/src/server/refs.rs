use std::sync::{Arc, Mutex, MutexGuard};
use crate::protocol::{PacketState, Protocol};
use crate::server::conn::ServerConnection;
use crate::server::Server;

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


pub struct ServerRef<S: PacketState, T: Protocol<S>> {
    pub(crate) server: Arc<Mutex<Server<S, T>>>
}

unsafe impl<S: PacketState, T: Protocol<S>> Send for ServerRef<S, T> {}
unsafe impl<S: PacketState, T: Protocol<S>> Sync for ServerRef<S, T> {}

impl<S: PacketState, T: Protocol<S>> Clone for ServerRef<S, T> {
    fn clone(&self) -> Self {
        ServerRef { server: self.server.clone() }
    }
}

impl<S: PacketState, T: Protocol<S>> ServerRef<S, T> {
    pub(crate) fn lock(&self) -> MutexGuard<'_, Server<S, T>> {
        self.server.lock().unwrap()
    }

    pub fn connections() -> Vec<ConnectionRef<S, T>> {
        todo!()
    }

    pub fn broadcast(packet: T) {
        todo!()
    }

    pub(crate) fn tmp_lock<R>(&self, f: fn(MutexGuard<'_, Server<S, T>>) -> R) -> R {
        f(self.lock())
    }
}