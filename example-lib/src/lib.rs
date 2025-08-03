pub mod my_module;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub fn random_number() -> u64 {
    use rand::Rng;
    let mut rng = rand::rng();
    rng.random_range(0..100)
}

#[cfg(test)]
mod tests {
    use regression_test::RegTest;
    use regression_test_macros::regtest;

    use super::*;

    #[regtest]
    fn it_works(mut r: RegTest) {
        let result = add(2, 2);
        assert_eq!(result, 4);
        r.regtest(result);
    }

    #[regtest]
    fn random_number_test(mut r: RegTest) {
        let result = random_number();
        assert!(result < 100);
        r.regtest(result);
    }
}
