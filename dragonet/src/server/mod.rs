mod conn;
mod refs;
mod events;

use std::alloc::System;
use std::any::Any;
use std::collections::HashMap;
use std::io::ErrorKind::{Interrupted, WouldBlock};
use std::io::{Read, Write};
use std::marker::PhantomData;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use mio::event::Event;
use crate::buffer::PacketBuf;
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
    connections: HashMap<Token, Arc<Mutex<ServerConnection<S, T>>>>,
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
        const SERVER_TOKEN: Token = Token(0);

        unsafe { CONNECTION_ID_COUNTER.fetch_add(1, Ordering::AcqRel); }

        let mut poll = Poll::new().unwrap();
        let mut events = Events::with_capacity(128);


        {
            let Some(socket) = &mut self.socket else {
                panic!("
    > ERROR
    | your server does not have an address configured
    | help: please use Server#with_address on your function labelled #[dragonet::server]
            ");
            };



            poll.registry()
                .register(socket, SERVER_TOKEN, Interest::READABLE | Interest::WRITABLE);
        }

        let rf = ServerRef { server: Arc::new(Mutex::new(self)) };

        for event in rf.clone().lock().startup_events.clone() {
            event(rf.clone());
        }
        
        loop {
            poll.poll(&mut events, None);

            println!("wow");

            for event in events.iter() {
                println!("Event: {:?}", event);
                match event.token() {
                    SERVER_TOKEN => Self::handle_server_token_event(rf.clone(), &poll, event),
                    token => Self::handle_other_token_event(rf.clone(), &poll, &token, event)
                }
            }
        }
    }

    fn handle_server_token_event(
        rf: ServerRef<S, T>,
        poll: &Poll,
        event: &Event
    ) {
        let crf = rf.clone();
        let mut drf = crf.lock();

        let (mut connection, address) = match drf.socket.as_mut().unwrap().accept() {
            Ok((connection, address)) => (connection, address),
            Err(e) if e.kind() == WouldBlock => { return; }
            Err(e) => { panic!("{}", e); }
        };

        let incremented = unsafe { CONNECTION_ID_COUNTER.fetch_add(1, Ordering::AcqRel) };
        let token = Token(incremented);

        poll.registry().register(
            &mut connection,
            token,
            Interest::READABLE | Interest::WRITABLE,
        ).unwrap();

        drf.connections.insert(
            token,
            Arc::new(Mutex::new(
                ServerConnection {
                    token,
                    stream: connection,
                    packet_queue: vec![],
                    state: None,
                    _phantom: PhantomData,
                }
            )),
        );

        for event in &drf.conn_events.clone() {
            let arc = drf.connections.get_mut(&token).expect("connection is guaranteed present here").clone();
            let reference = ConnectionRef { connection: arc };
            event(reference);
        }
    }

    fn handle_other_token_event(
        rf: ServerRef<S, T>,
        poll: &Poll,
        token: &Token,
        event: &Event
    ) {
        let crf = rf.clone();
        let mut drf = crf.lock();

        let events = drf.recv_events.clone();
        let tc = { drf.connections.get(token) };
        let done = if let Some(connection) = tc {
            Self::handle_connection_event(rf.clone(), connection.clone(), event, &events)
        } else {
            // ignore sporadic event
            false
        };
        if done {
            if let Some(mut connection) = drf.connections.remove(token) {
                poll.registry().deregister(&mut connection.lock().unwrap().stream).unwrap();
            }
        }
    }
}

