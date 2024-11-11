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

static mut CONNECTION_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);


pub struct ConnectionRef<S, T>
where
    S: PacketState,
    T: Protocol<S>,
{
    connection: Arc<Mutex<ServerConnection<S, T>>>,
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

pub struct ServerConnection<S, T>
where
    S: PacketState,
    T: Protocol<S>,
{
    token: Token,
    stream: TcpStream,
    packet_queue: Vec<T>,
    state: Option<S>,
    _phantom: PhantomData<(S, T)>,
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

pub struct Server<S, T>
where
    S: PacketState,
    T: Protocol<S>,
{
    socket: Option<TcpListener>,
    conn_events: Vec<fn(ConnectionRef<S, T>)>,
    recv_events: Vec<fn(ConnectionRef<S, T>, &T)>,
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
            _phantom: PhantomData,

        }
    }

    pub fn with_address(&mut self, addr: SocketAddrV4) -> &mut Server<S, T> {
        self.socket = Some(TcpListener::bind(SocketAddr::V4(addr)).unwrap());
        self
    }

    pub fn with_connection_event(&mut self, function: fn(ConnectionRef<S, T>)) -> &mut Server<S, T> {
        self.conn_events.push(function);
        self
    }

    pub fn with_packet_event(&mut self, function: fn(ConnectionRef<S, T>, &T)) -> &mut Server<S, T> {
        self.recv_events.push(function);
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

            for event in events.iter() {
                match event.token() {
                    SERVER_TOKEN => {
                        let (mut connection, address) = match socket.accept() {
                            Ok((connection, address)) => (connection, address),
                            Err(e) if e.kind() == WouldBlock => { break; }
                            Err(e) => { panic!("{}", e); }
                        };

                        let incremented = unsafe { CONNECTION_ID_COUNTER.fetch_add(1, Ordering::AcqRel) };
                        let token = Token(incremented);

                        poll.registry().register(
                            &mut connection,
                            token,
                            Interest::READABLE | Interest::WRITABLE,
                        ).unwrap();

                        self.connections.insert(
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

                        for event in &self.conn_events {
                            let arc = self.connections.get_mut(&token).expect("connection is guaranteed present here").clone();
                            let reference = ConnectionRef { connection: arc };
                            event(reference);
                        }
                    }
                    token => {
                        let events = self.recv_events.clone();
                        let tc = { self.connections.get(&token) };
                        let done = if let Some(connection) = tc {
                            Self::handle_connection_event(connection.clone(), event, &events)
                        } else {
                            // ignore sporadic event
                            false
                        };
                        if done {
                            if let Some(mut connection) = self.connections.remove(&token) {
                                poll.registry().deregister(&mut connection.lock().unwrap().stream).unwrap();
                            }
                        }
                    }
                }
            }
        }
    }

    fn handle_connection_event(
        connection: Arc<Mutex<ServerConnection<S, T>>>,
        event: &Event,
        events: &Vec<fn(ConnectionRef<S, T>, &T)>,
    ) -> bool {
        if event.is_readable() {
            Self::handle_connection_read(connection.clone(), event, events);
        }

        if event.is_writable() {
            Self::handle_connection_write(connection.clone(), event, events);
        }

        false
    }

    fn handle_connection_read(
        connection: Arc<Mutex<ServerConnection<S, T>>>,
        event: &Event,
        events: &Vec<fn(ConnectionRef<S, T>, &T)>,
    ) -> bool {
        let mut connection_closed = false;
        let mut data_buf = PacketBuf::with_capacity(1024);
        let mut bytes_read = 0;

        loop {
            let m = connection.clone().lock().unwrap().stream.read(data_buf.as_mut_array());
            match m {
                Ok(n) => {
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
                    return true;
                }
            }
        }

        if bytes_read != 0 {
            data_buf.resize(bytes_read);

            let length = data_buf.read_var_int();
            let id = data_buf.read_var_int();
            let meta = PacketMetadata {
                id: id as u32,
                state: connection.clone().lock().unwrap().state.clone().unwrap(),
                direction: PacketDirection::Serverbound,
            };
            let packet = T::decode(&mut data_buf, &meta);

            for event in events {
                let rf = ConnectionRef { connection: connection.clone() };
                event(rf, &packet);
            }
        }

        connection_closed
    }

    pub fn handle_connection_write(
        connection: Arc<Mutex<ServerConnection<S, T>>>,
        event: &Event,
        events: &Vec<fn(ConnectionRef<S, T>, &T)>,
    ) -> bool {
        
        /*
        let mut r = rf.lock();

        if !r.packet_queue.is_empty() {
            let packet = r.packet_queue.remove(0);
            let length = packet.size_of();
            let mut buf = PacketBuf::new();
            buf.write_var_int(length as i64);
            buf.write_var_int(packet.metadata().id as i64);
            buf.write_all(&packet.encode());
            r.socket.as_mut().unwrap().write_all(buf.as_mut_array()).unwrap();
        }
         */
        // write all packets in the queue per the specification
        
        let cl = connection.clone();
        let mut rf = cl.lock().unwrap();
        while !rf.packet_queue.is_empty() {
            let packet = rf.packet_queue.remove(0);
            let length = packet.size_of();
            let mut buf = PacketBuf::new();
            buf.write_var_int(length as i64);
            buf.write_var_int(packet.metadata().id as i64);
            buf.write_all(&packet.encode());
            rf.stream.write_all(buf.as_mut_array()).unwrap();
        }
        rf.packet_queue.clear();
        false
    }
}

