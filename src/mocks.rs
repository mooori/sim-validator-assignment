use crate::validator::RawValidatorData;

// TODO put this in a separate create that handles generating `RawValidatorData`.

/// Generates `num` instances of `RawValidatorData` each having the same `stake`. Out of theses
/// validators `num_malicious` are malicious.
pub fn new_validators(num: u64, stake: u64, num_malicious: u64) -> Vec<RawValidatorData> {
    let mut validators = vec![];
    for i in 0..num {
        let v = RawValidatorData {
            account_id: format!("validator_{i}"),
            stake,
            is_malicious: i < num_malicious,
        };
        validators.push(v)
    }
    validators
}

#[cfg(test)]
mod tests {
    use super::new_validators;

    #[test]
    fn test_new_validators() {
        insta::assert_yaml_snapshot!(new_validators(10, 100, 10 / 3));
    }
}
