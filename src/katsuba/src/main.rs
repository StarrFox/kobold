#![deny(
    rust_2018_idioms,
    rustdoc::broken_intra_doc_links,
    unsafe_op_in_unsafe_fn
)]

use clap::Parser;

mod cli;
use cli::Cli;

mod cmd;
use cmd::Command;

mod executor;

mod utils;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();
    cli.verbosity.setup();

    cli.command.handle()
}
