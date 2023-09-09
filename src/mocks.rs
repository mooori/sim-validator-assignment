use crate::validator::RawValidatorData;

// TODO put this in a separate create that handles generating `RawValidatorData`.

// TODO make `num_malicious_nodes` an input param.
pub fn new_100() -> Vec<RawValidatorData> {
    let mut validators = vec![];
    for i in 0..100 {
        let v = RawValidatorData {
            account_id: format!("validator_{i}"),
            stake: 10_000,
            is_malicious: i < 33,
        };
        validators.push(v)
    }
    validators
}

#[cfg(test)]
mod tests {
    use crate::mocks::new_100;

    #[test]
    fn test_new_100() {
        insta::assert_yaml_snapshot!(new_100());
    }
}
