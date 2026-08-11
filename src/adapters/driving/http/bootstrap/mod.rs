//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! HTTP bootstrap — server bring-up

pub mod port;
pub mod server;

pub use port::bind_with_retry;
pub use server::{run_server_with_config, ServerConfig};

#[cfg(test)]
mod port_tests;
