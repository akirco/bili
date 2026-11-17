//! `bili` command-line entry point.
//!
//! Everything lives in [`bili::cli::main_flow`] so it can be exercised from
//! integration tests; this file only parses arguments and sets the exit status.

use bili::cli::{Cli, main_flow};
use clap::Parser;

#[tokio::main]
async fn main() {
    std::process::exit(main_flow(Cli::parse()).await);
}
