use crate::config::Config;
use crate::partial_seat::{PartialSeat, ShuffledPartialSeats};
use crate::seat::ShuffledSeats;
use crate::shard::Shard;
use crate::validator::{
    new_ordered_partial_seats, new_ordered_seats, parse_raw_validator_data, RawValidatorData,
};
use num_rational::Ratio;
use num_traits::ToPrimitive;
use std::fs::read_to_string;
use std::path::Path;

pub fn run(config: &Config) -> anyhow::Result<()> {
    let raw_validator_data = match &config.validator_data {
        Some(file_path) => read_validator_data(file_path.as_path())?,
        None => mock_validator_data(),
    };

    let (population_stats, validators) = parse_raw_validator_data(config, &raw_validator_data);

    println!("population_stats: {:?}", population_stats);
    println!(
        "malicious_stake / stake ≈ {:.5}",
        Ratio::new(population_stats.malicious_stake, population_stats.stake)
            .to_f64()
            .unwrap()
    );
    println!(
        "malicious_seats / seats ≈ {:.5}",
        Ratio::new(population_stats.malicious_seats, population_stats.seats)
            .to_f64()
            .unwrap()
    );

    if population_stats.seats < u64::from(config.num_shards) * config.seats_per_shard {
        anyhow::bail!(
            "Validators cover {} seats, config requires {} seats",
            population_stats.seats,
            config.total_seats()
        )
    }

    let mut num_corrupted_shards = 0;

    for block_height in 0..config.num_blocks {
        let mut seats = new_ordered_seats(&validators);
        let shuffled_seats = ShuffledSeats::new(&mut seats);

        let mut partial_seats = config
            .include_partial_seats
            .then(|| new_ordered_partial_seats(&validators, config.stake_per_seat));
        let shuffled_partial_seats = partial_seats
            .as_mut()
            .map(|ps| ShuffledPartialSeats::new(ps));

        for shard_idx in 0..config.num_shards {
            let shard_idx = usize::from(shard_idx);
            let shard_seats =
                config.collect_seats_for_shard(shard_idx, shuffled_seats.get_seats())?;
            let shard_partial_seats =
                get_partial_seats_for_shard(config, shuffled_partial_seats.as_ref(), shard_idx)?;
            let shard = Shard::new(config, shard_seats, shard_partial_seats)?;
            if shard.is_corrupted(config) {
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

    println!(
        "Simulated {} blocks with {} shards each. The number of corrupted shards out of total shards is {} / {}",
        config.num_blocks, config.num_shards, num_corrupted_shards, config.num_blocks * u64::from(config.num_shards)
    );
    Ok(())
}

/// Reads validator data from a file exptected to contain `Vec<RawValidatorData>` serialized as
/// JSON.
fn read_validator_data(file_path: &Path) -> anyhow::Result<Vec<RawValidatorData>> {
    let file_content = read_to_string(file_path)?;
    serde_json::from_str::<Vec<RawValidatorData>>(&file_content).map_err(|err| err.into())
}

fn mock_validator_data() -> Vec<RawValidatorData> {
    // Mock a set of validators corresponding to the one used in Table 4 of this paper
    // https://www.montrealblockchainlab.com/New%20Mathematical%20Model.pdf
    // We model 1/3 of validators as malicious which corresponds to Class B (see Table 1).
    let num_validators = 4000;
    crate::mocks::new_validators(num_validators, 1, num_validators / 3)
}

fn log_heartbeat(block_height: u64, num_simulated_shards: u64, num_corrupted_shards: u64) {
    println!("heartbeat(block_height: {block_height}): {num_corrupted_shards} / {num_simulated_shards} shards corrupted");
}

fn get_partial_seats_for_shard<'a>(
    config: &'a Config,
    partial_seats: Option<&'a ShuffledPartialSeats>,
    shard_idx: usize,
) -> anyhow::Result<Option<Vec<&'a PartialSeat<'a>>>> {
    match partial_seats {
        Some(partial_seats) => {
            let shard_seats = config
                .collect_partial_seats_for_shard(shard_idx, partial_seats.get_partial_seats())?;
            Ok(Some(shard_seats))
        }
        None => Ok(None),
    }
}
