use crate::config::Config;
use crate::seat::Seat;
use num_rational::Ratio;

#[derive(Debug, Default)]
pub struct Shard<'seats> {
    seats: Vec<&'seats Seat<'seats>>,
    stake: u128,
    malicious_stake: u128,
}

impl<'seats> Shard<'seats> {
    pub fn new(config: &Config, seats: Vec<&'seats Seat>) -> anyhow::Result<Self> {
        if seats.len() != usize::try_from(config.seats_per_shard).unwrap() {
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
        shard.seats = seats;
        Ok(shard)
    }

    pub fn is_corrupted(&self, config: &Config) -> bool {
        Ratio::new(self.malicious_stake, self.stake) > config.max_malicious_stake_per_shard
    }
}
