use std::alloc::System;
use std::marker::PhantomData;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};
use crate::protocol::{PacketState, Protocol};

struct ServerConnection<S, T>
where
    S: PacketState,
    T: Protocol<S>,
{
    _phantom: PhantomData<(S, T)>,
}

pub struct Server<S, T>
where
    S: PacketState,
    T: Protocol<S>,
{
    socket: TcpListener,
    _phantom: PhantomData<(S, T)>,
}


impl<S: PacketState, T: Protocol<S>> Server<S, T> {
    pub fn new() -> Server<S, T> {
        Server {
            socket: TcpListener::bind(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 2000))).unwrap(),
            _phantom: PhantomData::default(),
        }
    }

    pub fn event_loop(&mut self) {
        const SERVER_TOKEN: Token = Token(0);

        let mut poll = Poll::new().unwrap();
        let mut events = Events::with_capacity(128);

        poll.registry()
            .register(&mut self.socket, SERVER_TOKEN, Interest::READABLE | Interest::WRITABLE);

        loop {
            poll.poll(&mut events, None);

            println!("{:?}", events);

            for event in events.iter() {
                println!("{:?}", event);
            }
        }
    }
}

