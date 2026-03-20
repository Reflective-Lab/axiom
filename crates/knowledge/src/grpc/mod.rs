//! gRPC service implementation.

#[allow(clippy::all)]
#[allow(missing_docs)]
mod knowledge {
    include!("converge.knowledge.v1.rs");
}

pub mod client;
pub mod server;

pub use client::KnowledgeClient;
pub use knowledge::*;
pub use server::KnowledgeServiceImpl;
