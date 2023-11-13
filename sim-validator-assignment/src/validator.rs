use num_rational::Ratio;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::partial_seat::PartialSeat;
use crate::seat::Seat;

#[derive(Deserialize, Serialize, Debug)]
pub struct RawValidatorData {
    pub account_id: String,
    pub stake: u128,
    pub is_malicious: bool,
}

pub fn parse_raw_validator_data(
    config: &Config,
    input: &[RawValidatorData],
) -> (PopulationStats, Vec<Validator>) {
    let mut population_stats = PopulationStats::default();
    let mut validators = vec![];

    for v in input.iter() {
        population_stats.stake += v.stake;
        let num_seats = config.seats_per_stake(v.stake);
        population_stats.seats += num_seats;
        if v.is_malicious {
            population_stats.malicious_stake += v.stake;
            population_stats.malicious_seats += num_seats;
        }
    }

    for v in input.iter() {
        validators.push(Validator {
            account_id: v.account_id.clone(),
            stake: v.stake,
            is_malicious: v.is_malicious,
            num_seats: config.seats_per_stake(v.stake),
            total_stake_share: Ratio::new(v.stake, population_stats.stake),
        })
    }

    (population_stats, validators)
}

impl From<dl_validator_data::ValidatorData> for RawValidatorData {
    /// The returned validator is malicious if `data.is_malicious == Some(true)`, otherwise it is
    /// not malicious.
    fn from(data: dl_validator_data::ValidatorData) -> Self {
        Self {
            account_id: data.account_id,
            stake: data.stake,
            is_malicious: data.is_malicious.is_some_and(|is_malicious| is_malicious),
        }
    }
}

/// Holds data describing a set of validators.
#[derive(Serialize, Default, Debug)]
pub struct PopulationStats {
    /// Sum of validator stakes.
    pub stake: u128,
    /// Sum of stake of malicious validator stakes.
    pub malicious_stake: u128,
    /// Total number of seats of all validators. Note that some stake might not participate in
    /// validation if it is not assigned to a seat. For example, if a validator has a stake of 17
    /// and seat price is 5, then the validator holds 3 seats (worth 15 stake) and 2 stake remain
    /// unassigned.
    pub seats: u64,
    /// The number of seats held by malicious validators.
    pub malicious_seats: u64,
}

#[derive(Serialize, PartialEq, Debug)]
pub struct Validator {
    account_id: String,
    stake: u128,
    is_malicious: bool,
    num_seats: u64,
    total_stake_share: Ratio<u128>,
}

impl Validator {
    /// Returns the validator's seats. The number of seats a validator claims is determined by the
    /// stake required per seat and the validator's stake.
    pub fn seats(&self) -> Vec<Seat> {
        let seat = Seat::new(self);
        vec![seat.clone(); self.num_seats_as_usize()]
    }

    /// Returns `num_seats` as usize and panics on overflow.
    ///
    /// Field `num_seats` is an unsigned integer in the struct to facilitate calculations.
    pub fn num_seats_as_usize(&self) -> usize {
        usize::try_from(self.num_seats).expect("num_seats should fit into usize")
    }

    /// Returns the validator's `PartialSeat`. It might have a weight of 0 if the validator's
    /// stake is 0 or there is no remainder after distributing the validator's stake to full seats.
    ///
    /// # Panics
    ///
    /// Panics if `stake_per_seat` is 0.
    pub fn partial_seat(&self, stake_per_seat: u128) -> PartialSeat {
        let weight = self.stake % stake_per_seat;
        PartialSeat::new(self, weight)
    }

    pub fn get_is_malicious(&self) -> bool {
        self.is_malicious
    }
}

pub fn new_ordered_seats(validators: &[Validator]) -> Vec<Seat> {
    // The `i`th element holds the seats of `validators[i]`
    let seats_per_validator: Vec<Vec<Seat>> = validators.iter().map(|v| v.seats()).collect();

    seats_per_validator.into_iter().flatten().collect()
}

/// Constructs `PartialSeats` for the provided `validators`. A validator whose partial seat has a
/// weight of zero will not be represented in the returned vector.
pub fn new_ordered_partial_seats(
    validators: &[Validator],
    stake_per_seat: u128,
) -> Vec<PartialSeat> {
    validators
        .iter()
        .filter_map(|v| {
            let partial_seat = v.partial_seat(stake_per_seat);
            if partial_seat.get_weight() > 0 {
                Some(partial_seat)
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
pub mod tests {
    use num_rational::Ratio;

    use crate::partial_seat::PartialSeat;
    use crate::validator::new_ordered_partial_seats;

    use super::Config;
    use super::RawValidatorData;
    use super::Validator;
    use super::{new_ordered_seats, parse_raw_validator_data};

    fn new_test_validator() -> Validator {
        Validator {
            account_id: "validator_0".to_owned(),
            stake: 300,
            is_malicious: false,
            num_seats: 3,
            total_stake_share: Ratio::new(3, 100),
        }
    }

    pub fn new_test_raw_validator_data() -> Vec<RawValidatorData> {
        vec![
            // Stake covers a full number of seats.
            RawValidatorData {
                account_id: "validator_0".to_owned(),
                stake: 500,
                is_malicious: false,
            },
            // Stake is elligible for seats and there is a remainder (partial seat).
            RawValidatorData {
                account_id: "validator_1".to_owned(),
                stake: 310,
                is_malicious: true,
            },
            // Stake is not sufficient for a seat.
            RawValidatorData {
                account_id: "validator_2".to_owned(),
                stake: 90,
                is_malicious: false,
            },
            // Some more validators, to have enough for running tests.
            RawValidatorData {
                account_id: "validator_3".to_owned(),
                stake: 100,
                is_malicious: true,
            },
            RawValidatorData {
                account_id: "validator_4".to_owned(),
                stake: 100,
                is_malicious: false,
            },
            RawValidatorData {
                account_id: "validator_5".to_owned(),
                stake: 100,
                is_malicious: false,
            },
            RawValidatorData {
                account_id: "validator_6".to_owned(),
                stake: 100,
                is_malicious: false,
            },
            RawValidatorData {
                account_id: "validator_7".to_owned(),
                stake: 100,
                is_malicious: false,
            },
            RawValidatorData {
                account_id: "validator_8".to_owned(),
                stake: 100,
                is_malicious: false,
            },
            RawValidatorData {
                account_id: "validator_9".to_owned(),
                stake: 100,
                is_malicious: false,
            },
            RawValidatorData {
                account_id: "validator_10".to_owned(),
                stake: 100,
                is_malicious: false,
            },
            RawValidatorData {
                account_id: "validator_11".to_owned(),
                stake: 100,
                is_malicious: false,
            },
        ]
    }

    #[test]
    fn test_new_ordered_seats() {
        assert_eq!(new_ordered_seats(&[]), vec![]);

        let config = Config::new_mock(false);
        let (_, validators) = parse_raw_validator_data(&config, &new_test_raw_validator_data());
        // Use a small set of validators to avoid bloating snapshot files.
        let validators = &validators[0..3];
        insta::with_settings!({
            info => &(
                config,
                "validators:",
                validators
            ),
        }, {
            insta::assert_yaml_snapshot!(new_ordered_seats(validators));
        })
    }

    #[test]
    fn test_parse_raw_validator_input() {
        let config = Config::new_mock(false);
        let (population_stats, validators) =
            parse_raw_validator_data(&config, &new_test_raw_validator_data());

        insta::with_settings!({
            info => &config,
        }, {
            insta::assert_yaml_snapshot!(population_stats);
            insta::assert_yaml_snapshot!(validators);
        })
    }

    #[test]
    fn test_validator_seats() {
        let mut validator_0_seats = new_test_validator();
        validator_0_seats.num_seats = 0;
        insta::assert_yaml_snapshot!(validator_0_seats.seats());

        let validator_3_seats = new_test_validator();
        insta::assert_yaml_snapshot!(validator_3_seats.seats());
    }

    #[test]
    fn test_validator_partial_seat() {
        let validator = new_test_validator();

        // stake fills full seats without remainder
        assert_eq!(validator.partial_seat(100), PartialSeat::new(&validator, 0));

        // there is a partial seat with `weight > 0`
        assert_eq!(validator.partial_seat(90), PartialSeat::new(&validator, 30));
    }

    #[test]
    fn test_new_ordered_partial_seats() {
        assert_eq!(new_ordered_partial_seats(&[], 1), vec![]);

        let config = Config::new_mock(true);
        let (_, validators) = parse_raw_validator_data(&config, &new_test_raw_validator_data());
        insta::with_settings!({
            info => &(
                &config,
                "validators:",
                &validators
            ),
        }, {
            insta::assert_yaml_snapshot!(new_ordered_partial_seats(&validators, config.stake_per_seat));
        })
    }
}
