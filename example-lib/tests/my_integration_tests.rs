use regression_test::RegTest;
use regression_test_macros::regtest;

#[regtest]
fn my_integration_test(mut r: RegTest) {
    // This is a simple integration test that uses the RegTest macro
    let result = example_lib::add(2, 3);
    assert_eq!(result, 5);
    r.regtest(result);

    let result2 = example_lib::add(3, 2);
    assert_eq!(result2, 5);
    r.regtest(result2);
}

#[regtest]
fn another_integration_test(mut r: RegTest) {
    // Another integration test that checks a random number
    let result = example_lib::random_number();
    assert!(result < 100);
    r.regtest(result);
}
