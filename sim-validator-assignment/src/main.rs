use clap::{Parser, Subcommand};

mod config;
use config::Config;
mod download;
use download::{download, DownloadConfig};
mod mocks;
mod partial_seat;
mod run;
mod seat;
mod seat_stats;
use seat_stats::{print_seat_stats, SeatStatsConfig};
mod shard;
use run::run;
mod validator;

/// A CLI to simulate blockchain validator assignments.
#[derive(Parser, Debug)]
#[command(name = "sim-validator-assignment")]
#[command(about = "A CLI to simulate blockchain validator assignments", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Runs a simulation
    #[command(arg_required_else_help = true)]
    Run(Config),
    /// Downloads valdiator data
    Download(DownloadConfig),
    /// Prints seat stats
    SeatStats(SeatStatsConfig),
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    match args.command {
        Command::Run(config) => run(&config),
        Command::Download(dl_config) => download(&dl_config),
        Command::SeatStats(ss_config) => print_seat_stats(&ss_config),
    }
}
