//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! HTTP bootstrap — server bring-up

pub mod server;

pub use server::{run_server_with_config, ServerConfig};
