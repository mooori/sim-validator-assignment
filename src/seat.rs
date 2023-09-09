use serde::Serialize;

use crate::validator::Validator;

/// Represents a seat filled by a particular validator. A seat may not outlive the validator it is
/// referrencing.
///
/// The stake corresponding to a seat is set by global configuration and is equal for every seat.
/// Hence, to avoid bloating this struct, it has no field or method to retrieve a seat's stake.
#[derive(Serialize, PartialEq, Debug, Clone)]
pub struct Seat<'validator> {
    validator: &'validator Validator,
}

impl<'validator> Seat<'validator> {
    pub fn new(validator: &'validator Validator) -> Self {
        Self { validator }
    }

    pub fn get_is_malicious(&self) -> bool {
        self.validator.get_is_malicious()
    }
}

pub struct ShuffledSeats<'seats> {
    seats: &'seats mut [Seat<'seats>],
}

// TODO impl new(seats) which shuffles them
impl<'seats> ShuffledSeats<'seats> {
    /// Shuffles the input `seats`.
    pub fn new(seats: &'seats mut [Seat<'seats>]) -> Self {
        // TODO mention library used for randomness in README
        fastrand::shuffle(seats);
        Self { seats }
    }

    pub fn get_seats(&self) -> &[Seat] {
        &self.seats
    }
}
