use std::alloc::System;
use std::collections::HashMap;
use std::io::ErrorKind::WouldBlock;
use std::marker::PhantomData;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use crate::protocol::{PacketState, Protocol};

static mut CONNECTION_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub struct ServerConnection<S, T>
where
    S: PacketState,
    T: Protocol<S>,
{
    token: Token,
    stream: TcpStream,
    _phantom: PhantomData<(S, T)>,
}

impl<S, T> ServerConnection<S, T>
where
    S: PacketState,
    T: Protocol<S>,
{
    pub fn send_packet(&mut self, packet: S) -> &mut ServerConnection<S, T> {
        // TODO: write to connection
        self
    }
}

pub struct Server<S, T>
where
    S: PacketState,
    T: Protocol<S>,
{
    socket: Option<TcpListener>,
    events: Vec<Box<dyn Fn(&ServerConnection<S, T>, &T)>>,
    connections: HashMap<Token, ServerConnection<S, T>>,
    _phantom: PhantomData<(S, T)>,
}


impl<S: PacketState, T: Protocol<S>> Server<S, T> {
    pub fn new() -> Server<S, T> {
        Server {
            socket: None,
            connections: HashMap::new(),
            events: Vec::new(),
            _phantom: PhantomData::default(),
        }
    }

    pub fn with_address(&mut self, addr: SocketAddrV4) -> &mut Server<S, T> {
        self.socket = Some(TcpListener::bind(SocketAddr::V4(addr)).unwrap());
        self
    }

    pub fn with_packet_event<F: Fn(&ServerConnection<S, T>, &T) + 'static>(&mut self, function: F) -> &mut Server<S, T> {
        self.events.push(Box::new(function));
        self
    }

    pub fn event_loop(&mut self) {
        const SERVER_TOKEN: Token = Token(0);
        
        unsafe { CONNECTION_ID_COUNTER.fetch_add(1, Ordering::AcqRel); }

        let mut poll = Poll::new().unwrap();
        let mut events = Events::with_capacity(128);

        let Some(socket) = &mut self.socket else {
            panic!("
    > ERROR
    | your server does not have an address configured
    | help: please use Server#with_address on your function labelled #[dragonet::server]
            ");
        };

        poll.registry()
            .register(socket, SERVER_TOKEN, Interest::READABLE | Interest::WRITABLE);

        loop {
            poll.poll(&mut events, None);

            println!("{:?}", events);

            for event in events.iter() {
                match event.token() {
                    SERVER_TOKEN => {
                        let (mut connection, address) = match socket.accept() {
                            Ok((connection, address)) => (connection, address),
                            Err(e) if e.kind() == WouldBlock => { break; },
                            Err(e) => { panic!("{}", e); }
                        };
                        let incremented = unsafe { CONNECTION_ID_COUNTER.fetch_add(1, Ordering::AcqRel) };
                        let token = Token(incremented);
                        poll.registry().register(
                            &mut connection,
                            token,
                            Interest::READABLE | Interest::WRITABLE
                        ).unwrap();
                        
                        println!("Accepted connection from {}", address);
                    }
                    token => {
                        let done = if let Some(connection) = self.connections.get_mut(&token) {
                            // logic here...
                            // logic should be extrapolated to a seperate function
                            // false is a placeholder
                            false
                        } else {
                            // ignore sporadic event
                            false
                        };
                        if done {
                            if let Some(mut connection) = self.connections.remove(&token) {
                                poll.registry().deregister(&mut connection.stream).unwrap();
                            }
                        }
                    }
                }
            }
        }
    }
}

