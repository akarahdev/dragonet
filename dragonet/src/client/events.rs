use std::io::ErrorKind::{Interrupted, WouldBlock};
use std::io::{Read, Write};
use mio::event::Event;
use crate::buffer::PacketBuf;
use crate::client::Client;
use crate::client::refs::ClientRef;
use crate::protocol::{PacketDirection, PacketMetadata, PacketState, Protocol};

impl<S: PacketState, T: Protocol<S>> Client<S, T> {
    pub(crate) fn handle_connection_event(
        rf: ClientRef<S, T>,
        event: &Event,
    ) -> bool {
        if event.is_readable() {
            Self::handle_connection_read(rf.clone(), event);
        }

        if event.is_writable() {
            Self::handle_connection_write(rf.clone(), event);
        }

        false
    }

    pub(crate) fn handle_connection_read(
        rf: ClientRef<S, T>,
        event: &Event,
    ) -> bool {
        let mut connection_closed = false;
        let mut data_buf = PacketBuf::with_capacity(1024);
        let mut bytes_read = 0;


        loop {
            let m = rf.lock().socket.as_mut().unwrap().read(data_buf.as_mut_array());
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
                    panic!("{}", err);
                }
            }
        }

        if bytes_read != 0 {
            data_buf.resize(bytes_read);

            let length = data_buf.read_var_int();
            let id = data_buf.read_var_int();
            let meta = PacketMetadata {
                id: id as u32,
                state: rf.lock().state.clone().unwrap(),
                direction: PacketDirection::Clientbound,
            };
            let packet = T::decode(&mut data_buf, &meta);
            let events = rf.lock().events.clone();

            for event in events {
                event(&rf.clone(), &packet);
            }
        }

        connection_closed
    }

    pub(crate) fn handle_connection_write(
        rf: ClientRef<S, T>,
        event: &Event,
    ) -> bool {
        // write all packets in the queue per the specification

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

        false
    }
}