use clap::{Parser, Subcommand};

mod wad;

// When not stuck with MSVC, use a more performant global
// allocator than the default one Rust uses.
#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Subcommand for working with KIWAD archives.
    Wad(wad::Wad),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Wad(wad) => wad::process(wad),
    }
}
