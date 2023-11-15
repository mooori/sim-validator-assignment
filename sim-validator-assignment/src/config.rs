use clap::Args;
use num_rational::Ratio;
use serde::Serialize;
use std::path::PathBuf;

use crate::{partial_seat::PartialSeat, seat::Seat};

#[derive(Args, Serialize, Debug)]
pub struct Config {
    #[arg(long)]
    pub num_blocks: u64,
    // Using `u16` because it allows infallible conversion to `usize` (which is not the case for
    // unsigned integer types with more bits, e.g. `u32`). For use cases of this simulation the
    // number of shards is expected to be less than `u16::MAX`.
    #[arg(long)]
    pub num_shards: u16,
    /// The set of validators must be sufficient to fill `num_shards * seats_per_shard` seats. Any
    /// seats above that threshold remain unassigned.
    #[arg(long)]
    pub seats_per_shard: u64,
    /// The amount of stake required to get one seat.
    #[arg(long)]
    pub stake_per_seat: u128,
    /// If the ratio of malicious stake is higher than this threshold, the shard is considered
    /// corrupted, i.e. a security failure occured.
    #[arg(long)]
    pub max_malicious_stake_per_shard: Ratio<u128>,
    /// The file from which validator data is read. It is expected to contain a vector of
    /// `RawValidatorData` serialized as JSON. If no validator data is provided, mocked validator
    /// data will be used in the simulation.
    #[arg(long)]
    pub validator_data: Option<PathBuf>,
    /// A validator's stake might not entirely cover seats given a particular `stake_per_seat`. This
    /// option controls whether remaining stake (not covering a full seat) should be assigned to a
    /// partial seat or ignored.
    #[arg(long, default_value_t = false)]
    pub include_partial_seats: bool,
}

/// Returns the number of (full) seats that can be claimed by `stake`.
pub fn seats_per_stake(stake: u128, stake_per_seat: u128) -> u64 {
    // Integer division in Rust returns the floor as described here
    // https://doc.rust-lang.org/std/primitive.u64.html#method.div_euclid
    u64::try_from(stake / stake_per_seat).expect("seats per stake should fit u64")
}

impl Config {
    #[cfg(test)]
    pub fn new_mock(include_partial_seats: bool) -> Self {
        Self {
            num_blocks: 1_000,
            num_shards: 4,
            seats_per_shard: 2,
            stake_per_seat: 100,
            max_malicious_stake_per_shard: Ratio::new(1, 3),
            validator_data: None,
            include_partial_seats,
        }
    }

    /// Returns the amount of seats for all shards that must be filled by validators.
    pub fn total_seats(&self) -> u64 {
        u64::from(self.num_shards)
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
        if shard_idx >= usize::from(self.num_shards) {
            anyhow::bail!(
                "shard_idx {} is an invalid index for {} shards",
                shard_idx,
                self.num_shards
            )
        }
        let required_seats =
            usize::from(self.num_shards) * usize::try_from(self.seats_per_shard).unwrap();
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

    /// Collect partials seats for `shard_idx` by picking seats from positions with `position %
    /// num_shards == shard_idx`.
    ///
    /// # Motivation for assignment via modulo
    ///
    /// Every validator may hold at most 1 partial seat. If a validator's stake covers full seats
    /// without remainder, it holds 0 partial seats. Hence `0 <= num_partial_seats <=
    /// num_validators`. Assigning seats to shards with `%` allows distributing the existing number
    /// of partial seats to shards as evenly as possible.
    ///
    /// # No minimum number of required _partial_ seats per shard
    ///
    /// It is not needed since partial seats are included only to distribute leftover stake (not
    /// covering full seats) to shards.
    pub fn collect_partial_seats_for_shard<'seats>(
        &self,
        shard_idx: usize,
        partial_seats: &'seats [PartialSeat],
    ) -> anyhow::Result<Vec<&PartialSeat<'seats>>> {
        if shard_idx >= usize::from(self.num_shards) {
            anyhow::bail!(
                "shard_idx {} is an invalid index for {} shards",
                shard_idx,
                self.num_shards
            )
        }

        let mut shard_partial_seats = vec![];
        for idx in (shard_idx..partial_seats.len()).step_by(self.num_shards.into()) {
            shard_partial_seats.push(&partial_seats[idx]);
        }

        Ok(shard_partial_seats)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{seats_per_stake, Config};
    use crate::validator::tests::new_test_raw_validator_data;
    use crate::validator::{
        new_ordered_partial_seats, new_ordered_seats, parse_raw_validator_data,
    };

    #[test]
    fn test_seats_per_stake() {
        let stake_per_seat = 100;
        assert_eq!(seats_per_stake(20, stake_per_seat), 0);
        assert_eq!(seats_per_stake(100, stake_per_seat), 1);
        assert_eq!(seats_per_stake(530, stake_per_seat), 5);
    }

    #[test]
    fn test_total_seats() {
        let config = Config::new_mock(false);
        assert_eq!(config.total_seats(), 8);
    }

    #[test]
    fn test_collect_seats_for_shard() {
        let config = Config::new_mock(false);
        let (_, validators) =
            parse_raw_validator_data(&new_test_raw_validator_data(), config.stake_per_seat);
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
    fn test_collect_seats_for_shard_errors() {
        let config = Config::new_mock(false);
        let (_, validators) =
            parse_raw_validator_data(&new_test_raw_validator_data(), config.stake_per_seat);
        let seats = new_ordered_seats(&validators);

        insta::assert_debug_snapshot!(config.collect_seats_for_shard(4, &seats));
        insta::assert_debug_snapshot!(config.collect_seats_for_shard(0, &[]));
    }

    #[test]
    fn test_collect_partial_seats_for_shard() {
        let mut config = Config::new_mock(true);
        config.stake_per_seat = 90;
        let (_, validators) =
            parse_raw_validator_data(&new_test_raw_validator_data(), config.stake_per_seat);
        // Using ordered partial seats as input to have a deterministic result of
        // `collect_partial_seats_for_shard`.
        let partial_seats = new_ordered_partial_seats(&validators, config.stake_per_seat);

        // Using `BTreeMap` for deterministic ordering of keys.
        let mut assignments = BTreeMap::new();
        for shard_idx in 0..config.num_shards {
            let assignment = config
                .collect_partial_seats_for_shard(shard_idx.into(), &partial_seats)
                .unwrap();
            assignments.insert(format!("shard_{shard_idx}"), assignment);
        }

        insta::with_settings!({
            info => &(
                &config,
                "partial_seats:",
                &partial_seats
            )
        }, {
            insta::assert_yaml_snapshot!(assignments);
        })
    }

    #[test]
    fn test_collect_partial_seats_for_shard_errors() {
        let config = Config::new_mock(true);
        let (_, validators) =
            parse_raw_validator_data(&new_test_raw_validator_data(), config.stake_per_seat);
        let partial_seats = new_ordered_partial_seats(&validators, config.stake_per_seat);

        insta::assert_debug_snapshot!(
            config.collect_partial_seats_for_shard(config.num_shards.into(), &partial_seats)
        );
    }
}
