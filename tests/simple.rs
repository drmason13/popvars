use cases::read_test_case;

mod cases;

#[test]
fn simple() -> Result<(), Box<dyn std::error::Error>> {
    let test_case = read_test_case("simple.md")?;
    test_case.run()?;
    Ok(())
}
