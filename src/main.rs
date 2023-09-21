use clap::Parser;

mod config;
use config::Config;
mod mocks;
mod seat;
use seat::ShuffledSeats;
mod shard;
use shard::Shard;
mod validator;
use validator::{new_ordered_seats, parse_raw_validator_data};

fn main() -> anyhow::Result<()> {
    let config = Config::parse();

    // Mock a set of validators corresponding to the one used in Table 4 of this paper
    // https://www.montrealblockchainlab.com/New%20Mathematical%20Model.pdf
    // We model 1/3 of validators as malicious which corresponds to Class B (see Table 1).
    let num_validators = 4000;
    let raw_validator_data = mocks::new_validators(num_validators, 1, num_validators / 3);

    let (population_stats, validators) = parse_raw_validator_data(&config, &raw_validator_data);

    println!("population_stats: {:?}", population_stats);
    if population_stats.seats < u64::from(config.num_shards) * config.seats_per_shard {
        anyhow::bail!(
            "Validators cover {} seats, config requires {} seats",
            population_stats.seats,
            config.total_seats()
        )
    }

    let mut num_corrupted_shards = 0;

    for block_height in 0..config.num_blocks {
        let mut ordered_seats = new_ordered_seats(&validators);
        let shuffled_seats = ShuffledSeats::new(&mut ordered_seats);

        for shard_idx in 0..config.num_shards {
            let shard_idx = usize::from(shard_idx);
            let shard_seats =
                config.collect_seats_for_shard(shard_idx, shuffled_seats.get_seats())?;
            let shard = Shard::new(&config, shard_seats)?;
            if shard.is_corrupted(&config) {
                num_corrupted_shards += 1;
            }
        }

        if block_height % 100_000 == 0 {
            log_heartbeat(
                block_height,
                block_height * u64::from(config.num_shards),
                num_corrupted_shards,
            );
        }
    }

    println!("Simulated {} blocks with {} shards each. The number of corrupted shards out of total shards is {} / {}",
    config.num_blocks, config.num_shards, num_corrupted_shards, config.num_blocks * u64::from(config.num_shards)
);
    Ok(())
}

fn log_heartbeat(block_height: u64, num_simulated_shards: u64, num_corrupted_shards: u64) {
    println!("heartbeat(block_height: {block_height}): {num_corrupted_shards} / {num_simulated_shards} shards corrupted");
}
