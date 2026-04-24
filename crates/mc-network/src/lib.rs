//! Client-server networking: packet protocol, chat commands, and TCP server.
//!
//! Defines [`ClientPacket`]/[`ServerPacket`] with length-framed encoding,
//! a slash-command parser ([`parse_command`]), and [`GameServer`] for multiplayer hosting.

pub mod client_conn;
pub mod command_help;
pub mod commands;
pub mod latency_meter;
pub mod packet;
pub mod protocol;
pub mod server;

pub use client_conn::ClientConnection;
pub use commands::{Command, CommandError, CommandResult, parse_command};
pub use packet::{ClientPacket, DiggingStatus, ServerPacket};
pub use protocol::{
    ProtocolError, decode_client, decode_server, encode_client, encode_server, frame, unframe,
};
pub use server::{GameServer, ServerConfig};
