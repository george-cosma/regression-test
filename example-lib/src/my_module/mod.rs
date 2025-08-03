pub fn subtract(left: u64, right: u64) -> u64 {
    left - right
}

mod tests {
    use regression_test::RegTest;
    use regression_test_macros::regtest;

    use super::*;

    #[regtest]
    fn it_subtracts_correctly(mut r: RegTest) {
        let result = subtract(5, 3);
        assert_eq!(result, 2);
        r.regtest(result);
    }
}
