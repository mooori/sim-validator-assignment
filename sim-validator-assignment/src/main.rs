use clap::Parser;

mod config;
use config::Config;
mod mocks;
mod run;
mod seat;
mod shard;
use run::run;
mod validator;

fn main() -> anyhow::Result<()> {
    let config = Config::parse();
    run(&config)
}
