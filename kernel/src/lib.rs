pub mod dag;
pub mod execution;
pub mod io;
pub mod notebook;
pub mod server;
pub mod mcp;

pub mod ai;

pub mod proto {
    include!("opennotebook.v1.rs");
}
