use serde::Serialize;

use crate::validator::Validator;

/// Represents a partial seat filled by a particular validator.
///
/// A partial seat may not outlive the validator it is referrencing.
#[derive(Serialize, PartialEq, Debug, Clone)]
pub struct PartialSeat<'validator> {
    /// Reference to the validator holding this particular partial seat.
    validator: &'validator Validator,
    /// The stake attributed to the partial seat. Note that `0 < weight < stake_per_seat`.
    ///
    /// For example, let validator `V` have a stake of 12 and let `stake_per_seat = 5`. Then `V`
    /// holds 2 full seats and a partial seat with `weight = 2`.
    weight: u128,
}

impl<'validator> PartialSeat<'validator> {
    /// Constructs the partial seat filled by `validator`.
    pub fn new(validator: &'validator Validator, weight: u128) -> Self {
        Self { validator, weight }
    }

    pub fn get_is_malicious(&self) -> bool {
        self.validator.get_is_malicious()
    }

    pub fn get_weight(&self) -> u128 {
        self.weight
    }
}

pub struct ShuffledPartialSeats<'seats> {
    partial_seats: &'seats [PartialSeat<'seats>],
}

impl<'seats> ShuffledPartialSeats<'seats> {
    /// Shuffles the input `partial_seats`.
    // TODO(rand) do all shuffling operations in one generic fn
    pub fn new(partial_seats: &'seats mut [PartialSeat<'seats>]) -> Self {
        fastrand::shuffle(partial_seats);
        Self { partial_seats }
    }

    pub fn get_partial_seats(&self) -> &[PartialSeat] {
        self.partial_seats
    }
}
