use clap::Args;
use num_rational::Ratio;
use num_traits::ToPrimitive;
use std::path::PathBuf;

use crate::{
    config::seats_per_stake,
    partial_seat::PartialSeat,
    validator::{
        new_ordered_partial_seats, parse_raw_validator_data, read_validator_data, Validator,
    },
};

#[derive(Args, Debug)]
pub(crate) struct SeatStatsConfig {
    /// The amount of stake required to get one seat.
    #[arg(long)]
    pub stake_per_seat: u128,
    /// The file from which validator data is read. It is expected to contain a vector of
    /// `RawValidatorData` serialized as JSON.
    #[arg(long)]
    pub validator_data: PathBuf,
    /// Whether to print stats for partial seats.
    #[arg(long, default_value_t = false)]
    pub include_partial_seats: bool,
}

pub(crate) fn print_seat_stats(config: &SeatStatsConfig) -> anyhow::Result<()> {
    let raw_validator_data = read_validator_data(config.validator_data.as_path())?;
    let (population_stats, validators) =
        parse_raw_validator_data(&raw_validator_data, config.stake_per_seat);

    println!(
        "stake/malicious_stake\t{:.4}",
        Ratio::new(population_stats.stake, population_stats.malicious_stake)
            .to_f64()
            .unwrap(),
    );
    println!("num_seats\t{}", population_stats.seats);
    println!("num_malicious_seats\t{}", population_stats.malicious_seats);
    println!(
        "malicious_seats/seats\t{}",
        Ratio::new(population_stats.malicious_seats, population_stats.seats)
            .to_f64()
            .unwrap()
    );

    if config.include_partial_seats {
        print_partial_seat_stats(&config, &validators);
    }

    Ok(())
}

fn print_partial_seat_stats(config: &SeatStatsConfig, validators: &[Validator]) {
    let partial_seats = new_ordered_partial_seats(validators, config.stake_per_seat);
    let malicious_partial_seats = partial_seats
        .iter()
        .filter(|ps| ps.get_is_malicious())
        .cloned()
        .collect::<Vec<PartialSeat>>();
    let sum_weights = sum_partial_seat_weights(&partial_seats);
    let sum_malicious_weights = sum_partial_seat_weights(&malicious_partial_seats);

    println!("num_partial_seats\t{}", partial_seats.len());
    println!(
        "num_malicious_partial_seats\t{}",
        malicious_partial_seats.len()
    );
    println!(
        "equivalent_num_seats\t{}",
        seats_per_stake(sum_weights, config.stake_per_seat)
    );
    println!(
        "equivalent_num_malicious_seats\t{}",
        seats_per_stake(sum_malicious_weights, config.stake_per_seat)
    );
}

fn sum_partial_seat_weights(partial_seats: &[PartialSeat]) -> u128 {
    partial_seats
        .iter()
        .fold(0, |acc, ps| acc + ps.get_weight())
}
