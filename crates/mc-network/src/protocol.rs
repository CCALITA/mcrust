use std::fmt;

use crate::packet::{ClientPacket, ServerPacket};

/// Errors that can occur during protocol encoding/decoding.
#[derive(Debug)]
pub enum ProtocolError {
    SerializeFailed(String),
    DeserializeFailed(String),
    InvalidPacket,
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SerializeFailed(msg) => write!(f, "serialize failed: {msg}"),
            Self::DeserializeFailed(msg) => write!(f, "deserialize failed: {msg}"),
            Self::InvalidPacket => write!(f, "invalid packet"),
        }
    }
}

impl std::error::Error for ProtocolError {}

// ---------------------------------------------------------------------------
// Encode / Decode
// ---------------------------------------------------------------------------

/// Serialize a `ClientPacket` to bytes using bincode.
pub fn encode_client(packet: &ClientPacket) -> Result<Vec<u8>, ProtocolError> {
    bincode::serialize(packet).map_err(|e| ProtocolError::SerializeFailed(e.to_string()))
}

/// Deserialize bytes into a `ClientPacket`.
pub fn decode_client(bytes: &[u8]) -> Result<ClientPacket, ProtocolError> {
    bincode::deserialize(bytes).map_err(|e| ProtocolError::DeserializeFailed(e.to_string()))
}

/// Serialize a `ServerPacket` to bytes using bincode.
pub fn encode_server(packet: &ServerPacket) -> Result<Vec<u8>, ProtocolError> {
    bincode::serialize(packet).map_err(|e| ProtocolError::SerializeFailed(e.to_string()))
}

/// Deserialize bytes into a `ServerPacket`.
pub fn decode_server(bytes: &[u8]) -> Result<ServerPacket, ProtocolError> {
    bincode::deserialize(bytes).map_err(|e| ProtocolError::DeserializeFailed(e.to_string()))
}

// ---------------------------------------------------------------------------
// Framing helpers (length-prefixed)
// ---------------------------------------------------------------------------

/// Prepend a 4-byte little-endian length header to `data`.
pub fn frame(data: Vec<u8>) -> Vec<u8> {
    let len = data.len() as u32;
    let mut buf = Vec::with_capacity(4 + data.len());
    buf.extend_from_slice(&len.to_le_bytes());
    buf.extend_from_slice(&data);
    buf
}

/// Read a length-prefixed frame from `data`.
///
/// Returns `(payload, bytes_consumed)` on success.
pub fn unframe(data: &[u8]) -> Result<(&[u8], usize), ProtocolError> {
    if data.len() < 4 {
        return Err(ProtocolError::InvalidPacket);
    }
    let len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    let total = 4 + len;
    if data.len() < total {
        return Err(ProtocolError::InvalidPacket);
    }
    Ok((&data[4..total], total))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packet::DiggingStatus;

    // ---- Client packets round-trip -----------------------------------------

    #[test]
    fn roundtrip_client_handshake() {
        let pkt = ClientPacket::Handshake {
            protocol_version: 42,
            player_name: "Alex".into(),
        };
        let decoded = decode_client(&encode_client(&pkt).unwrap()).unwrap();
        match decoded {
            ClientPacket::Handshake {
                protocol_version,
                player_name,
            } => {
                assert_eq!(protocol_version, 42);
                assert_eq!(player_name, "Alex");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn roundtrip_client_player_position() {
        let pkt = ClientPacket::PlayerPosition {
            x: 1.0,
            y: 64.0,
            z: -3.5,
            on_ground: true,
        };
        let decoded = decode_client(&encode_client(&pkt).unwrap()).unwrap();
        match decoded {
            ClientPacket::PlayerPosition { x, y, z, on_ground } => {
                assert_eq!(x, 1.0);
                assert_eq!(y, 64.0);
                assert_eq!(z, -3.5);
                assert!(on_ground);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn roundtrip_client_player_look() {
        let pkt = ClientPacket::PlayerLook {
            yaw: 90.0,
            pitch: -45.0,
        };
        let decoded = decode_client(&encode_client(&pkt).unwrap()).unwrap();
        match decoded {
            ClientPacket::PlayerLook { yaw, pitch } => {
                assert_eq!(yaw, 90.0);
                assert_eq!(pitch, -45.0);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn roundtrip_client_player_digging() {
        let pkt = ClientPacket::PlayerDigging {
            pos: (10, 20, 30),
            face: 2,
            status: DiggingStatus::Finished,
        };
        let decoded = decode_client(&encode_client(&pkt).unwrap()).unwrap();
        match decoded {
            ClientPacket::PlayerDigging { pos, face, status } => {
                assert_eq!(pos, (10, 20, 30));
                assert_eq!(face, 2);
                assert_eq!(status, DiggingStatus::Finished);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn roundtrip_client_block_place() {
        let pkt = ClientPacket::PlayerBlockPlace {
            pos: (5, 60, -1),
            face: 1,
            block_id: 4,
        };
        let decoded = decode_client(&encode_client(&pkt).unwrap()).unwrap();
        match decoded {
            ClientPacket::PlayerBlockPlace {
                pos,
                face,
                block_id,
            } => {
                assert_eq!(pos, (5, 60, -1));
                assert_eq!(face, 1);
                assert_eq!(block_id, 4);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn roundtrip_client_chat() {
        let pkt = ClientPacket::ChatMessage {
            message: "hello".into(),
        };
        let decoded = decode_client(&encode_client(&pkt).unwrap()).unwrap();
        match decoded {
            ClientPacket::ChatMessage { message } => assert_eq!(message, "hello"),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn roundtrip_client_keepalive() {
        let pkt = ClientPacket::KeepAlive { id: 999 };
        let decoded = decode_client(&encode_client(&pkt).unwrap()).unwrap();
        match decoded {
            ClientPacket::KeepAlive { id } => assert_eq!(id, 999),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn roundtrip_client_disconnect() {
        let pkt = ClientPacket::Disconnect;
        let decoded = decode_client(&encode_client(&pkt).unwrap()).unwrap();
        assert!(matches!(decoded, ClientPacket::Disconnect));
    }

    // ---- Server packets round-trip -----------------------------------------

    #[test]
    fn roundtrip_server_login_success() {
        let pkt = ServerPacket::LoginSuccess { player_id: 7 };
        let decoded = decode_server(&encode_server(&pkt).unwrap()).unwrap();
        match decoded {
            ServerPacket::LoginSuccess { player_id } => assert_eq!(player_id, 7),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn roundtrip_server_chunk_data() {
        let pkt = ServerPacket::ChunkData {
            cx: 0,
            cz: -1,
            sections: vec![(0, vec![1, 2, 3])],
        };
        let decoded = decode_server(&encode_server(&pkt).unwrap()).unwrap();
        match decoded {
            ServerPacket::ChunkData { cx, cz, sections } => {
                assert_eq!(cx, 0);
                assert_eq!(cz, -1);
                assert_eq!(sections, vec![(0, vec![1, 2, 3])]);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn roundtrip_server_block_change() {
        let pkt = ServerPacket::BlockChange {
            x: 1,
            y: 2,
            z: 3,
            block_id: 10,
        };
        let decoded = decode_server(&encode_server(&pkt).unwrap()).unwrap();
        match decoded {
            ServerPacket::BlockChange { x, y, z, block_id } => {
                assert_eq!((x, y, z, block_id), (1, 2, 3, 10));
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn roundtrip_server_entity_spawn() {
        let pkt = ServerPacket::EntitySpawn {
            entity_id: 100,
            entity_type: 5,
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        let decoded = decode_server(&encode_server(&pkt).unwrap()).unwrap();
        match decoded {
            ServerPacket::EntitySpawn {
                entity_id,
                entity_type,
                x,
                y,
                z,
            } => {
                assert_eq!(entity_id, 100);
                assert_eq!(entity_type, 5);
                assert_eq!((x, y, z), (1.0, 2.0, 3.0));
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn roundtrip_server_entity_move() {
        let pkt = ServerPacket::EntityMove {
            entity_id: 50,
            dx: 0.5,
            dy: -1.0,
            dz: 0.0,
        };
        let decoded = decode_server(&encode_server(&pkt).unwrap()).unwrap();
        match decoded {
            ServerPacket::EntityMove {
                entity_id,
                dx,
                dy,
                dz,
            } => {
                assert_eq!(entity_id, 50);
                assert_eq!((dx, dy, dz), (0.5, -1.0, 0.0));
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn roundtrip_server_entity_despawn() {
        let pkt = ServerPacket::EntityDespawn { entity_id: 77 };
        let decoded = decode_server(&encode_server(&pkt).unwrap()).unwrap();
        match decoded {
            ServerPacket::EntityDespawn { entity_id } => assert_eq!(entity_id, 77),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn roundtrip_server_player_position_and_look() {
        let pkt = ServerPacket::PlayerPositionAndLook {
            x: 0.0,
            y: 64.0,
            z: 0.0,
            yaw: 180.0,
            pitch: 0.0,
        };
        let decoded = decode_server(&encode_server(&pkt).unwrap()).unwrap();
        match decoded {
            ServerPacket::PlayerPositionAndLook {
                x,
                y,
                z,
                yaw,
                pitch,
            } => {
                assert_eq!((x, y, z), (0.0, 64.0, 0.0));
                assert_eq!((yaw, pitch), (180.0, 0.0));
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn roundtrip_server_time_update() {
        let pkt = ServerPacket::TimeUpdate { time_of_day: 0.25 };
        let decoded = decode_server(&encode_server(&pkt).unwrap()).unwrap();
        match decoded {
            ServerPacket::TimeUpdate { time_of_day } => assert_eq!(time_of_day, 0.25),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn roundtrip_server_chat() {
        let pkt = ServerPacket::ChatMessage {
            sender: "Admin".into(),
            message: "Welcome!".into(),
        };
        let decoded = decode_server(&encode_server(&pkt).unwrap()).unwrap();
        match decoded {
            ServerPacket::ChatMessage { sender, message } => {
                assert_eq!(sender, "Admin");
                assert_eq!(message, "Welcome!");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn roundtrip_server_keepalive() {
        let pkt = ServerPacket::KeepAlive { id: 123456 };
        let decoded = decode_server(&encode_server(&pkt).unwrap()).unwrap();
        match decoded {
            ServerPacket::KeepAlive { id } => assert_eq!(id, 123456),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn roundtrip_server_disconnect() {
        let pkt = ServerPacket::Disconnect {
            reason: "Server shutting down".into(),
        };
        let decoded = decode_server(&encode_server(&pkt).unwrap()).unwrap();
        match decoded {
            ServerPacket::Disconnect { reason } => {
                assert_eq!(reason, "Server shutting down");
            }
            _ => panic!("wrong variant"),
        }
    }

    // ---- Framing -----------------------------------------------------------

    #[test]
    fn frame_and_unframe_roundtrip() {
        let data = vec![1, 2, 3, 4, 5];
        let framed = frame(data.clone());
        assert_eq!(framed.len(), 4 + 5);

        let (payload, consumed) = unframe(&framed).unwrap();
        assert_eq!(payload, &data[..]);
        assert_eq!(consumed, 9);
    }

    #[test]
    fn frame_empty_payload() {
        let framed = frame(vec![]);
        let (payload, consumed) = unframe(&framed).unwrap();
        assert!(payload.is_empty());
        assert_eq!(consumed, 4);
    }

    #[test]
    fn unframe_truncated_header() {
        let result = unframe(&[0x01, 0x00]);
        assert!(result.is_err());
    }

    #[test]
    fn unframe_truncated_payload() {
        // Header says 10 bytes but only 2 provided after header.
        let mut data = 10u32.to_le_bytes().to_vec();
        data.extend_from_slice(&[0xAA, 0xBB]);
        let result = unframe(&data);
        assert!(result.is_err());
    }

    #[test]
    fn decode_client_garbage_returns_error() {
        let result = decode_client(&[0xFF, 0xFF, 0xFF]);
        assert!(result.is_err());
    }

    #[test]
    fn decode_server_garbage_returns_error() {
        let result = decode_server(&[0xFF, 0xFF, 0xFF]);
        assert!(result.is_err());
    }

    #[test]
    fn frame_decode_integration_client() {
        let pkt = ClientPacket::KeepAlive { id: 42 };
        let framed = frame(encode_client(&pkt).unwrap());
        let (payload, consumed) = unframe(&framed).unwrap();
        assert_eq!(consumed, framed.len());
        let decoded = decode_client(payload).unwrap();
        assert!(matches!(decoded, ClientPacket::KeepAlive { id: 42 }));
    }

    #[test]
    fn frame_decode_integration_server() {
        let pkt = ServerPacket::TimeUpdate { time_of_day: 0.5 };
        let framed = frame(encode_server(&pkt).unwrap());
        let (payload, _consumed) = unframe(&framed).unwrap();
        let decoded = decode_server(payload).unwrap();
        match decoded {
            ServerPacket::TimeUpdate { time_of_day } => assert_eq!(time_of_day, 0.5),
            _ => panic!("wrong variant"),
        }
    }
}
