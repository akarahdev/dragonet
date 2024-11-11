use std::io::ErrorKind::{Interrupted, WouldBlock};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use mio::event::Event;
use crate::buffer::PacketBuf;
use crate::protocol::{PacketDirection, PacketMetadata, PacketState, Protocol};
use crate::server::conn::ServerConnection;
use crate::server::refs::ConnectionRef;
use crate::server::Server;

impl<S: PacketState, T: Protocol<S>> Server<S, T> {
    pub(crate) fn handle_connection_event(
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

    pub(crate) fn handle_connection_read(
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