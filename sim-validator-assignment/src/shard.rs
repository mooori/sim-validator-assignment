use crate::config::Config;
use crate::partial_seat::PartialSeat;
use crate::seat::Seat;
use num_rational::Ratio;

#[derive(Debug, Default)]
pub struct Shard<'seats> {
    seats: Vec<&'seats Seat<'seats>>,
    partial_seats: Vec<&'seats PartialSeat<'seats>>,
    stake: u128,
    malicious_stake: u128,
}

impl<'seats> Shard<'seats> {
    pub fn new(
        config: &Config,
        seats: Vec<&'seats Seat>,
        partial_seats: Vec<&'seats PartialSeat>,
    ) -> anyhow::Result<Self> {
        if seats.len() != usize::try_from(config.seats_per_shard).unwrap() {
            // Count only _full_ seats for the minimum number of required seats, since it is not
            // clear how a _partial_ seat should be weighted for that concern.
            // Validator assignment frameworks might try to minimize the number of partial seats or
            // try to ignore them entirely.
            anyhow::bail!(
                "Shard requires {} seats, received {} seats",
                config.seats_per_shard,
                seats.len()
            )
        }

        let mut shard = Self::default();
        for s in seats.iter() {
            shard.stake += config.stake_per_seat;
            if s.get_is_malicious() {
                shard.malicious_stake += config.stake_per_seat;
            }
        }

        for ps in partial_seats.iter() {
            let weight = ps.get_weight();
            shard.stake += weight;
            if ps.get_is_malicious() {
                shard.malicious_stake += weight;
            }
        }

        shard.seats = seats;
        shard.partial_seats = partial_seats;
        Ok(shard)
    }

    pub fn is_corrupted(&self, config: &Config) -> bool {
        Ratio::new(self.malicious_stake, self.stake) > config.max_malicious_stake_per_shard
    }
}
