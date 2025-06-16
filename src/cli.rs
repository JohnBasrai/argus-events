//! Command-line argument parser for Argus Events server.

use clap::Parser;

/// Command-line options for configuring the server.
#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Args {
    /// Address to bind to (host:port). Can also be set via ARGUS_ENDPOINT.
    #[arg(long, env = "ARGUS_ENDPOINT", default_value = "0.0.0.0:3000")]
    pub endpoint: String,

    /// Storage backend to use (e.g., memory, redis). Can also be set via ARGUS_REPOSITORY.
    #[arg(long, env = "ARGUS_REPOSITORY", default_value = "memory")]
    pub repository: String,
}
