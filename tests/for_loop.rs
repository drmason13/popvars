use cases::read_test_case;

mod cases;

#[test]
fn single_for_loop() -> Result<(), Box<dyn std::error::Error>> {
    let test_case = read_test_case("for-loop.md")?;
    test_case.run()?;
    Ok(())
}
