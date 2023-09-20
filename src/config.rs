use clap::Parser;
use num_rational::Ratio;
use serde::Serialize;

use crate::seat::Seat;

// TODO change stake related types to u128
#[derive(Parser, Serialize, Debug)]
pub struct Config {
    #[arg(long)]
    pub num_blocks: u64,
    // TODO change to `u16`
    #[arg(long)]
    pub num_shards: u64,
    /// TODO mention excess validators are ignored/unassigned.
    #[arg(long)]
    pub seats_per_shard: u64,
    /// The amount of stake required to get one seat.
    #[arg(long)]
    pub stake_per_seat: u64,
    /// If the ratio of malicious stake is higher than this threshold, the shard is considered
    /// corrupted, i.e. a security failure occured.
    #[arg(long)]
    pub max_malicious_stake_per_shard: Ratio<u64>,
}

impl Config {
    #[cfg(test)]
    pub fn new_mock() -> Self {
        Self {
            num_blocks: 1_000,
            num_shards: 4,
            seats_per_shard: 2,
            stake_per_seat: 100,
            max_malicious_stake_per_shard: Ratio::new(1, 3),
        }
    }

    /// Returns the number of (full) seats that can be claimed by `stake`.
    pub fn seats_per_stake(&self, stake: u64) -> u64 {
        // Integer division in Rust returns the floor as described here
        // https://doc.rust-lang.org/std/primitive.u64.html#method.div_euclid
        stake / self.stake_per_seat
    }

    /// Returns the amount of seats for all shards that must be filled by validators.
    pub fn total_seats(&self) -> u64 {
        self.num_shards
            .checked_mul(self.seats_per_shard)
            .expect("min_required_seats should fit into return type")
    }

    /// Collects the (consecutive) seats required for `shard_idx` starting from `seats[shard_idx *
    /// self.seats_per_shard]`.
    ///
    /// # Panics
    ///
    /// If `shard_idx` is out of bounds or if `seats` contains not enough seats.
    pub fn collect_seats_for_shard<'seats>(
        &self,
        shard_idx: usize,
        seats: &'seats [Seat],
    ) -> anyhow::Result<Vec<&Seat<'seats>>> {
        if u64::try_from(shard_idx).unwrap() >= self.num_shards {
            anyhow::bail!(
                "shard_idx {} is an invalid index for {} shards",
                shard_idx,
                self.num_shards
            )
        }
        let required_seats = usize::try_from(self.num_shards * self.seats_per_shard).unwrap();
        if seats.len() < required_seats {
            anyhow::bail!(
                "validators fill only {}/{} of seats",
                seats.len(),
                required_seats
            );
        }

        let seats_per_shard = usize::try_from(self.seats_per_shard).unwrap();
        let start = shard_idx * seats_per_shard;
        let shard_seats: Vec<_> = seats[start..start + seats_per_shard].iter().collect();

        Ok(shard_seats)
    }
}

#[cfg(test)]
mod tests {
    use super::Config;
    use crate::validator::tests::new_test_raw_validator_data;
    use crate::validator::{new_ordered_seats, parse_raw_validator_data};

    #[test]
    pub fn test_seats_per_stake() {
        let config = Config::new_mock();
        assert_eq!(config.seats_per_stake(20), 0);
        assert_eq!(config.seats_per_stake(100), 1);
        assert_eq!(config.seats_per_stake(530), 5);
    }

    #[test]
    pub fn test_total_seats() {
        let config = Config::new_mock();
        assert_eq!(config.total_seats(), 8);
    }

    #[test]
    pub fn test_collect_seats_for_shard() {
        let config = Config::new_mock();
        let (_, validators) = parse_raw_validator_data(&config, &new_test_raw_validator_data());
        // Using ordered seats as input to have a deterministic result of `collect_seats_for_shard`.
        let seats = new_ordered_seats(&validators);

        insta::with_settings!({
            info => &(
                &config,
                "seats:",
                &seats
            )
        }, {
            insta::assert_yaml_snapshot!(config.collect_seats_for_shard(0, &seats).unwrap());
            insta::assert_yaml_snapshot!(config.collect_seats_for_shard(1, &seats).unwrap());
            insta::assert_yaml_snapshot!(config.collect_seats_for_shard(2, &seats).unwrap());
            insta::assert_yaml_snapshot!(config.collect_seats_for_shard(3, &seats).unwrap());
        })
    }

    #[test]
    pub fn test_collect_seats_for_shard_errors() {
        let config = Config::new_mock();
        let (_, validators) = parse_raw_validator_data(&config, &new_test_raw_validator_data());
        let seats = new_ordered_seats(&validators);

        insta::assert_debug_snapshot!(config.collect_seats_for_shard(4, &seats));
        insta::assert_debug_snapshot!(config.collect_seats_for_shard(0, &[]));
    }
}
