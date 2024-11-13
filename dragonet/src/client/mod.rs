mod refs;
mod events;

use std::any::Any;
use std::collections::HashMap;
use std::io::ErrorKind::{Interrupted, WouldBlock};
use std::io::{Read, Write};
use std::marker::PhantomData;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::ops::Deref;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use mio::event::Event;
use crate::buffer::Buffer;
use crate::client::refs::ClientRef;
use crate::protocol::{PacketDirection, PacketMetadata, PacketState, Protocol};

type ClientPacketEvent<S, T> = fn(ClientRef<S, T>, &T);

pub struct Client<S, T>
where
    S: PacketState,
    T: Protocol<S>,
{
    socket: Option<TcpStream>,
    events: Vec<ClientPacketEvent<S, T>>,
    on_connection: fn(ClientRef<S, T>),
    packet_queue: Vec<T>,
    state: Option<S>,
    _phantom: PhantomData<(S, T)>,
}




impl<S: PacketState, T: Protocol<S>> Default for Client<S, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: PacketState, T: Protocol<S>> Client<S, T> {
    pub fn new() -> Client<S, T> {
        Client {
            socket: None,
            events: Vec::new(),
            on_connection: |_| {},
            packet_queue: vec![],
            state: None,
            _phantom: PhantomData,
        }
    }

    pub fn with_address(&mut self, addr: SocketAddrV4) -> &mut Client<S, T> {
        self.socket = Some(TcpStream::connect(SocketAddr::V4(addr)).unwrap());
        self
    }

    pub fn with_packet_event(&mut self, function: ClientPacketEvent<S, T>) -> &mut Client<S, T> {
        self.events.push(function);
        self
    }

    pub fn on_connect(&mut self, function: fn(ClientRef<S, T>)) -> &mut Client<S, T> {
        self.on_connection = function;
        self
    }

    pub fn event_loop(mut self) {
        const SERVER_TOKEN: Token = Token(0);

        let mut poll = Poll::new().unwrap();
        let mut events = Events::with_capacity(128);

        let Some(socket) = &mut self.socket else {
            panic!("
    > ERROR
    | your client does not have an address configured
    | help: please use Client#with_address on your function labelled #[dragonet::client]
            ");
        };

        poll.registry()
            .register(socket, SERVER_TOKEN, Interest::READABLE | Interest::WRITABLE);


        let rf = ClientRef {
            client: Arc::new(Mutex::new(self)),
        };

        

        let mut had_writing_opportunity = false;
        
        loop {
            poll.poll(&mut events, None);

            for event in events.iter() {
                println!("wow {:?}", event);

                if event.is_writable() && !had_writing_opportunity {
                    let func = {
                        rf.lock().on_connection
                    };
                    func(rf.clone());
                }
                match event.token() {
                    SERVER_TOKEN => {
                        if Self::handle_connection_event(rf.clone(), event) {
                            return;
                        }
                    }
                    _ => panic!("unknown token")
                }
            }
        }
    }
}

