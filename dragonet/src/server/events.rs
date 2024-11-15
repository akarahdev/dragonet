use std::io::ErrorKind::{Interrupted, WouldBlock};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use mio::event::Event;
use crate::buffer::Buffer;
use crate::protocol::{PacketDirection, PacketMetadata, PacketState, Protocol};
use crate::server::conn::ServerConnection;
use crate::server::refs::{ConnectionRef, ServerRef};
use crate::server::{Server, ServerPacketEvent};

impl<S: PacketState, T: Protocol<S>> Server<S, T> {
    pub(crate) fn handle_connection_event(
        rf: ServerRef<S, T>,
        connection: Arc<Mutex<ServerConnection<S, T>>>,
        event: &Event,
        events: &[ServerPacketEvent<S, T>],
    ) -> bool {
        if event.is_readable() {
            Self::handle_connection_read(rf.clone(), connection.clone(), event, events);
        }

        if event.is_writable() {
            Self::handle_connection_write(rf.clone(), connection.clone(), event, events);
        }

        false
    }

    pub(crate) fn handle_connection_read(
        rf: ServerRef<S, T>,
        connection: Arc<Mutex<ServerConnection<S, T>>>,
        event: &Event,
        events: &[ServerPacketEvent<S, T>],
    ) -> bool {
        let mut connection_closed = false;
        let mut data_buf = Buffer::with_capacity(1024);
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
            let meta = match connection.clone().lock().unwrap().state.clone() {
                Some(s) => PacketMetadata {
                    id: id as u32,
                    state: s,
                    direction: PacketDirection::Serverbound,
                },
                None => panic!("server must give the client a valid state")
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
        rf: ServerRef<S, T>,
        connection: Arc<Mutex<ServerConnection<S, T>>>,
        event: &Event,
        events: &[ServerPacketEvent<S, T>],
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
            let tmp_buffer = packet.encode();
            let length = tmp_buffer.length();
            let mut buf = Buffer::new();
            buf.write_var_int(length as i64);
            buf.write_var_int(packet.metadata().id as i64);
            buf.write_all(&tmp_buffer);
            rf.stream.write_all(buf.as_mut_array()).unwrap();
        }
        rf.packet_queue.clear();
        rf.stream.flush();

        false
    }
}