#[cfg(test)]
extern crate fluid;

extern crate env_logger;
extern crate log;
extern crate lsp_types;
extern crate nom;
extern crate nom_locate;
extern crate serde;
extern crate serde_json;
extern crate tokio;

mod codec;
mod model;
mod parser;
pub mod server;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
