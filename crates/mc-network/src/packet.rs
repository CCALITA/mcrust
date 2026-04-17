use serde::{Deserialize, Serialize};

/// Status of a block-digging action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiggingStatus {
    Started,
    Cancelled,
    Finished,
}

/// Packets sent from the client to the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientPacket {
    Handshake {
        protocol_version: u32,
        player_name: String,
    },
    PlayerPosition {
        x: f32,
        y: f32,
        z: f32,
        on_ground: bool,
    },
    PlayerLook {
        yaw: f32,
        pitch: f32,
    },
    PlayerDigging {
        pos: (i32, i32, i32),
        face: u8,
        status: DiggingStatus,
    },
    PlayerBlockPlace {
        pos: (i32, i32, i32),
        face: u8,
        block_id: u16,
    },
    ChatMessage {
        message: String,
    },
    KeepAlive {
        id: u64,
    },
    Disconnect,
}

/// Packets sent from the server to the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerPacket {
    LoginSuccess {
        player_id: u64,
    },
    /// Chunk column data: section index + flat block-id array per section.
    ChunkData {
        cx: i32,
        cz: i32,
        sections: Vec<(u8, Vec<u16>)>,
    },
    BlockChange {
        x: i32,
        y: i32,
        z: i32,
        block_id: u16,
    },
    EntitySpawn {
        entity_id: u64,
        entity_type: u16,
        x: f32,
        y: f32,
        z: f32,
    },
    EntityMove {
        entity_id: u64,
        dx: f32,
        dy: f32,
        dz: f32,
    },
    EntityDespawn {
        entity_id: u64,
    },
    PlayerPositionAndLook {
        x: f32,
        y: f32,
        z: f32,
        yaw: f32,
        pitch: f32,
    },
    TimeUpdate {
        time_of_day: f32,
    },
    ChatMessage {
        sender: String,
        message: String,
    },
    KeepAlive {
        id: u64,
    },
    Disconnect {
        reason: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_packet_handshake_roundtrip() {
        let pkt = ClientPacket::Handshake {
            protocol_version: 1,
            player_name: "Steve".into(),
        };
        let bytes = bincode::serialize(&pkt).unwrap();
        let decoded: ClientPacket = bincode::deserialize(&bytes).unwrap();
        match decoded {
            ClientPacket::Handshake {
                protocol_version,
                player_name,
            } => {
                assert_eq!(protocol_version, 1);
                assert_eq!(player_name, "Steve");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn server_packet_chunk_data_roundtrip() {
        let pkt = ServerPacket::ChunkData {
            cx: -3,
            cz: 7,
            sections: vec![(0, vec![1, 2, 3]), (4, vec![10, 20])],
        };
        let bytes = bincode::serialize(&pkt).unwrap();
        let decoded: ServerPacket = bincode::deserialize(&bytes).unwrap();
        match decoded {
            ServerPacket::ChunkData { cx, cz, sections } => {
                assert_eq!(cx, -3);
                assert_eq!(cz, 7);
                assert_eq!(sections.len(), 2);
                assert_eq!(sections[0], (0, vec![1, 2, 3]));
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn digging_status_serialize_roundtrip() {
        for status in [
            DiggingStatus::Started,
            DiggingStatus::Cancelled,
            DiggingStatus::Finished,
        ] {
            let bytes = bincode::serialize(&status).unwrap();
            let decoded: DiggingStatus = bincode::deserialize(&bytes).unwrap();
            assert_eq!(decoded, status);
        }
    }
}
