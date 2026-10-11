// lib.rs — public facade. All implementation lives in msglink.rs.
pub mod msglink;

pub use msglink::{
    Client, ClientHandler, Ctx, Error, Handler, Message, Result, Server, Token,
    SESSION_TTL_SECS,
}; 