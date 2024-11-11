use std::sync::{Arc, Mutex, MutexGuard};
use crate::client::Client;
use crate::protocol::{PacketState, Protocol};

pub struct ClientRef<S, T>
where
    S: PacketState,
    T: Protocol<S>,
{
    pub(crate) client: Arc<Mutex<Client<S, T>>>,
}

impl<S, T> Clone for ClientRef<S, T>
where
    S: PacketState,
    T: Protocol<S>,
{
    fn clone(&self) -> Self {
        ClientRef {
            client: self.client.clone()
        }
    }
}

impl<S, T> ClientRef<S, T>
where
    S: PacketState,
    T: Protocol<S>,
{
    pub fn set_state(&self, state: S) {
        self.client.lock().unwrap().state = Some(state);
    }

    pub fn send_packet(&self, packet: T) {
        self.client.lock().unwrap().packet_queue.push(packet);
    }

    pub(crate) fn lock(&self) -> MutexGuard<'_, Client<S, T>> {
        self.client.lock().unwrap()
    }
}