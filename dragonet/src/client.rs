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
use crate::buffer::PacketBuf;
use crate::protocol::{PacketDirection, PacketMetadata, PacketState, Protocol};

pub struct Client<S, T>
where
    S: PacketState,
    T: Protocol<S>,
{
    socket: Option<TcpStream>,
    events: Vec<fn(&ClientRef<S, T>, &T)>,
    on_connection: fn(ClientRef<S, T>),
    packet_queue: Vec<T>,
    state: Option<S>,
    _phantom: PhantomData<(S, T)>,
}

pub struct ClientRef<'a, S, T>
where
    S: PacketState,
    T: Protocol<S>,
{
    client: Arc<Mutex<&'a mut Client<S, T>>>,
}

impl<'a, S, T> Clone for ClientRef<'a, S, T>
where
    S: PacketState,
    T: Protocol<S> {
    fn clone(&self) -> Self {
        ClientRef {
            client: self.client.clone()
        }
    }
}

impl<'a, S, T> ClientRef<'a, S, T>
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

    fn lock(&self) -> MutexGuard<'_, &'a mut Client<S, T>> {
        self.client.lock().unwrap()
    }
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

    pub fn with_packet_event(&mut self, function: fn(&ClientRef<S, T>, &T)) -> &mut Client<S, T> {
        self.events.push(function);
        self
    }

    pub fn on_connect(&mut self, function: fn(ClientRef<S, T>)) -> &mut Client<S, T> {
        self.on_connection = function;
        self
    }

    pub fn event_loop(&mut self) {
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

        println!("got here a");


        let init_ref = ClientRef {
            client: Arc::new(Mutex::new(self)),
        };

        let func = {
            println!("got here b");
            init_ref.lock().on_connection.clone()
        };
        func(init_ref.clone());

        println!("got here c");

        loop {
            poll.poll(&mut events, None);

            println!("events: {:?}", events);

            for event in events.iter() {
                match event.token() {
                    SERVER_TOKEN => {
                        if self.handle_connection_event(event) {
                           return;
                        }
                    },
                    _ => panic!("unknown token")
                }
            }
        }
    }

    fn handle_connection_event(
        &mut self,
        event: &Event,
    ) -> bool {
        if event.is_readable() {
            self.handle_connection_read(event);
        }

        if event.is_writable() {
            self.handle_connection_write(event);
        }

        false
    }

    fn handle_connection_read(
        &mut self,
        event: &Event,
    ) -> bool {
        let mut connection_closed = false;
        let mut data_buf = PacketBuf::with_capacity(1024);
        let mut bytes_read = 0;

        loop {
            let m = self.socket.as_mut().unwrap().read(data_buf.as_mut_array());
            println!("{:?}", m);
            match m {
                Ok(n) => {
                    println!("read client data: {}", n);
                    if n == 0 {
                        connection_closed = true;
                        break;
                    }
                    bytes_read += n;
                    data_buf.resize(data_buf.length() + bytes_read * 2);
                }
                Err(ref err) => {
                    if err.kind() == WouldBlock {
                        break;
                    }
                    if err.kind() == Interrupted {
                        continue;
                    }
                    panic!("{}", err);
                }
            }
        }

        println!("bytes read: {}", bytes_read);

        if bytes_read != 0 {
            data_buf.resize(bytes_read);

            let length = data_buf.read_var_int();
            let id = data_buf.read_var_int();
            let meta = PacketMetadata {
                id: id as u32,
                state: self.state.clone().unwrap(),
                direction: PacketDirection::Clientbound,
            };
            let packet = T::decode(&mut data_buf, &meta);
            let events = self.events.clone();
            let refer = ClientRef {
                client: Arc::new(Mutex::new(self))
            };

            for event in events {
                event(&refer, &packet);
            }
            println!("{:?}", data_buf);
        }

        connection_closed
    }

    pub fn handle_connection_write(
        &mut self,
        event: &Event,
    ) -> bool {
        // write all packets in the queue per the specification
        for packet in &self.packet_queue {
            let length = packet.size_of();
            let mut buf = PacketBuf::new();
            buf.write_var_int(length as i64);
            buf.write_var_int(packet.metadata().id as i64);
            buf.write_all(&packet.encode());
            self.socket.as_mut().unwrap().write_all(buf.as_array());
        }
        self.packet_queue.clear();
        false
    }
}

