# Dragonet

[crates-badge]: https://img.shields.io/crates/v/dragonet.svg
[crates-url]: https://crates.io/crates/dragonet

Dragonet is a (work in progress) Rust networking library.
It allows you to easily implement your own protocol, handling
all the networking and states for you - you just need to
glue it together with functionality.

Note that Dragonet uses sync Rust as opposed to async to allow for more
simplicity. The performance cost is low though due to thread pooling and
non-blocking and queueing IO operations as opposed to instantly performing them.

## Dependency Justification
| Dependency | Justification                                                         |
|------------|-----------------------------------------------------------------------|
| mio        | Non-blocking IO library used in the backend for the client and server |

## Example
```rust
protocol! {
    state Chat {
        clientbound packet ChatMessage;
        serverbound packet ChatMessage;
    }
}

#[dragonet::client]
fn client_provider(client: &mut Client<ProtocolState, Packets>) -> &mut Client<ProtocolState, Packets> {
    client
        .with_address(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 2000))
        .on_connect(|clr| {
            println!("Started connection");
            clr.set_state(ProtocolState::Chat);
            clr.send_packet(Packets::C2SChatMessage);
        })
        .with_packet_event(|connection, packet| {
            match packet {
                Packets::S2CChatMessage => {
                    println!("Received a message!");
                    std::thread::spawn(move || {
                        std::thread::sleep(Duration::from_millis(1000));
                        connection.send_packet(Packets::C2SChatMessage);
                    }).join().unwrap();
                }
                p => panic!("got serverbound packet somehow! {:?}", p)
            }
        })
}

#[dragonet::server]
fn server_provider(server: &mut Server<ProtocolState, Packets>) -> &mut Server<ProtocolState, Packets> {
    server
        .with_address(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 2000))
        .with_startup_event(|server| {
            println!("Starting!");
        })
        .with_connection_event(|connection| {
            println!("Started connection");
            connection.set_state(ProtocolState::Chat);
        })
        .with_packet_event(|connection, packet| {
            match packet {
                Packets::C2SChatMessage => {
                    println!("Received chat message on server!");
                    std::thread::spawn(move || {
                        std::thread::sleep(Duration::from_millis(1000));
                        connection.send_packet(Packets::S2CChatMessage);
                    }).join().unwrap();
                }
                _ => panic!("got clientbound packet somehow!")
            }
        })
}
```
