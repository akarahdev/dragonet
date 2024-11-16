mod conn;
mod refs;

use std::alloc::System;
use std::any::Any;
use std::collections::HashMap;
use std::io::ErrorKind::{Interrupted, WouldBlock};
use std::io::{Read, Write};
use std::marker::PhantomData;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, SocketAddrV6, TcpListener};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Duration;
use crate::buffer::Buffer;
use crate::protocol::{PacketDirection, PacketMetadata, PacketState, Protocol};
use crate::server::conn::ServerConnection;
use crate::server::refs::{ConnectionRef, ServerRef};

static mut CONNECTION_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

type ServerPacketEvent<S, T> = fn(ConnectionRef<S, T>, &T);



pub struct Server<S, T>
where
    S: PacketState,
    T: Protocol<S>,
{
    socket: Option<TcpListener>,
    conn_events: Vec<fn(ConnectionRef<S, T>)>,
    recv_events: Vec<ServerPacketEvent<S, T>>,
    startup_events: Vec<fn(ServerRef<S, T>)>,
    connections: HashMap<usize, Arc<Mutex<ServerConnection<S, T>>>>,
    _phantom: PhantomData<(S, T)>,
}


impl<S: PacketState, T: Protocol<S>> Default for Server<S, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: PacketState, T: Protocol<S>> Server<S, T> {
    pub fn new() -> Server<S, T> {
        Server {
            socket: None,
            connections: HashMap::new(),
            conn_events: Vec::new(),
            recv_events: Vec::new(),
            startup_events: Vec::new(),
            _phantom: PhantomData,
        }
    }

    pub fn with_address(&mut self, addr: SocketAddrV4) -> &mut Server<S, T> {
        self.socket = Some(TcpListener::bind(SocketAddr::V4(addr)).unwrap());
        self
    }

    pub fn with_startup_event(&mut self, function: fn(ServerRef<S, T>)) -> &mut Server<S, T> {
        self.startup_events.push(function);
        self
    }

    pub fn with_connection_event(&mut self, function: fn(ConnectionRef<S, T>)) -> &mut Server<S, T> {
        self.conn_events.push(function);
        self
    }

    pub fn with_packet_event(&mut self, function: ServerPacketEvent<S, T>) -> &mut Server<S, T> {
        self.recv_events.push(function);
        self
    }

    pub fn event_loop(mut self) {

    }
}

