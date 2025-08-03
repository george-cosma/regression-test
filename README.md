# regression-test

This crate provides you with the tools to write regression tests for your Rust
code. It can either be used directly as a library or through the
`regression-test-macros` crate, which provides convenient macros for writing
tests.

Under the hood, it uses `serde` for serialization and deserialization of
test data. When first run, it will create a JSON file containing the
expected output of the test. On subsequent runs, it will compare the actual
output of the test with the expected output in the JSON file. 

To regenerate the expected output, simply delete the JSON file and run the test again.

If you are using the `regression-test-macros` crate, you can use the
`#[regtest]` attribute macro to mark your test functions. This macro will
automatically handle the initialization of the `RegTest` struct and will place
the generated data in an appropriate place.

## Usage

```rust
use regression_test::RegTest;

#[test]
fn my_test() {
    let mut rt = RegTest::new("./my_test.json");

    let result = 2 + 2; // complex calculation

    // For structures that implement `Display`:
    rt.regtest(result);

    // For structures that implement `Debug`:
    rt.regtest_dbg(result);
}
```

You can also use the `regression-test-macros` crate to simplify your tests:

```rust
use regression_test_macros::regtest;
use regression_test::RegTest;

// Note: you must have the first argument of the function be a `RegTest` instance
#[regtest]
fn my_test(mut rt: RegTest) {
    let result = 2 + 2; // complex calculation

    // For structures that implement `Display`:
    rt.regtest(result);

    // For structures that implement `Debug`:
    rt.regtest_dbg(result);
}