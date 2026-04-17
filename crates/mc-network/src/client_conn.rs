use std::io::{self, Read, Write};
use std::net::TcpStream;

use crate::packet::{ClientPacket, ServerPacket};
use crate::protocol::{decode_client, encode_server, frame, unframe};

/// A single client's TCP connection and associated player state.
pub struct ClientConnection {
    pub id: u64,
    pub stream: TcpStream,
    pub player_name: String,
    pub position: (f32, f32, f32),
    pub yaw: f32,
    pub pitch: f32,
    pub authenticated: bool,
    recv_buffer: Vec<u8>,
    disconnect_reason: Option<String>,
}

impl ClientConnection {
    /// Create a new client connection, setting the stream to non-blocking mode.
    pub fn new(id: u64, stream: TcpStream) -> io::Result<Self> {
        stream.set_nonblocking(true)?;
        Ok(Self {
            id,
            stream,
            player_name: String::new(),
            position: (0.0, 64.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
            authenticated: false,
            recv_buffer: Vec::new(),
            disconnect_reason: None,
        })
    }

    /// Read available bytes from the stream and decode any complete packets.
    ///
    /// Returns all fully-framed packets that were available. Returns an empty
    /// vec when no complete packets are ready (including `WouldBlock`).
    pub fn try_read(&mut self) -> Vec<ClientPacket> {
        let mut tmp = [0u8; 4096];
        loop {
            match self.stream.read(&mut tmp) {
                Ok(0) => {
                    // Peer closed the connection.
                    self.disconnect_reason = Some("connection closed by peer".into());
                    break;
                }
                Ok(n) => {
                    self.recv_buffer.extend_from_slice(&tmp[..n]);
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(e) => {
                    self.disconnect_reason = Some(format!("read error: {e}"));
                    break;
                }
            }
        }

        let mut packets = Vec::new();
        while let Ok((payload, consumed)) = unframe(&self.recv_buffer) {
            if let Ok(packet) = decode_client(payload) {
                packets.push(packet);
            }
            // Remove the consumed bytes from the front of the buffer.
            self.recv_buffer = self.recv_buffer[consumed..].to_vec();
        }

        packets
    }

    /// Encode and send a server packet to this client.
    pub fn send(&mut self, packet: &ServerPacket) -> io::Result<()> {
        let encoded = encode_server(packet);
        let framed = frame(encoded);
        self.stream.write_all(&framed)
    }

    /// Returns the disconnect reason if the connection has been lost.
    pub fn disconnect_reason(&self) -> Option<&str> {
        self.disconnect_reason.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    use crate::protocol::{encode_client, frame as proto_frame};

    /// Helper: create a connected pair of streams via a loopback listener.
    fn connected_pair() -> (TcpStream, TcpStream) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let client_stream = TcpStream::connect(addr).unwrap();
        let (server_stream, _) = listener.accept().unwrap();
        (server_stream, client_stream)
    }

    #[test]
    fn new_sets_defaults() {
        let (server_stream, _client_stream) = connected_pair();
        let conn = ClientConnection::new(42, server_stream).unwrap();
        assert_eq!(conn.id, 42);
        assert_eq!(conn.player_name, "");
        assert_eq!(conn.position, (0.0, 64.0, 0.0));
        assert!(!conn.authenticated);
        assert!(conn.disconnect_reason().is_none());
    }

    #[test]
    fn try_read_returns_empty_when_no_data() {
        let (server_stream, _client_stream) = connected_pair();
        let mut conn = ClientConnection::new(1, server_stream).unwrap();
        let packets = conn.try_read();
        assert!(packets.is_empty());
    }

    #[test]
    fn try_read_decodes_single_packet() {
        let (server_stream, mut client_stream) = connected_pair();
        let mut conn = ClientConnection::new(1, server_stream).unwrap();

        // Write a framed handshake packet from the "client" side.
        let pkt = ClientPacket::Handshake {
            protocol_version: 1,
            player_name: "Steve".into(),
        };
        let data = proto_frame(encode_client(&pkt));
        client_stream.write_all(&data).unwrap();
        client_stream.flush().unwrap();

        // Give the OS a moment to deliver bytes.
        std::thread::sleep(std::time::Duration::from_millis(50));

        let packets = conn.try_read();
        assert_eq!(packets.len(), 1);
        match &packets[0] {
            ClientPacket::Handshake {
                protocol_version,
                player_name,
            } => {
                assert_eq!(*protocol_version, 1);
                assert_eq!(player_name, "Steve");
            }
            other => panic!("expected Handshake, got {:?}", other),
        }
    }

    #[test]
    fn try_read_decodes_multiple_packets() {
        let (server_stream, mut client_stream) = connected_pair();
        let mut conn = ClientConnection::new(1, server_stream).unwrap();

        let pkt1 = ClientPacket::KeepAlive { id: 10 };
        let pkt2 = ClientPacket::ChatMessage {
            message: "hello".into(),
        };
        let mut data = proto_frame(encode_client(&pkt1));
        data.extend_from_slice(&proto_frame(encode_client(&pkt2)));
        client_stream.write_all(&data).unwrap();
        client_stream.flush().unwrap();

        std::thread::sleep(std::time::Duration::from_millis(50));

        let packets = conn.try_read();
        assert_eq!(packets.len(), 2);
        assert!(matches!(packets[0], ClientPacket::KeepAlive { id: 10 }));
        assert!(matches!(&packets[1], ClientPacket::ChatMessage { message } if message == "hello"));
    }

    #[test]
    fn send_delivers_packet() {
        let (server_stream, mut client_stream) = connected_pair();
        client_stream.set_nonblocking(false).unwrap();
        let mut conn = ClientConnection::new(1, server_stream).unwrap();

        let pkt = ServerPacket::LoginSuccess { player_id: 42 };
        conn.send(&pkt).unwrap();

        // Read the framed data from the client side.
        let mut buf = [0u8; 256];
        let n = client_stream.read(&mut buf).unwrap();
        assert!(n > 0);

        // Unframe and decode.
        let (payload, _) = unframe(&buf[..n]).unwrap();
        let decoded: ServerPacket = bincode::deserialize(payload).unwrap();
        match decoded {
            ServerPacket::LoginSuccess { player_id } => assert_eq!(player_id, 42),
            other => panic!("expected LoginSuccess, got {:?}", other),
        }
    }

    #[test]
    fn disconnect_reason_set_on_peer_close() {
        let (server_stream, client_stream) = connected_pair();
        let mut conn = ClientConnection::new(1, server_stream).unwrap();
        drop(client_stream);

        std::thread::sleep(std::time::Duration::from_millis(50));

        let _ = conn.try_read();
        assert!(conn.disconnect_reason().is_some());
    }
}
