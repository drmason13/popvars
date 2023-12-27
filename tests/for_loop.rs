use cases::read_test_case;

mod cases;

#[test]
fn single_for_loop() -> Result<(), Box<dyn std::error::Error>> {
    let test_case = read_test_case("for-loop.md")?;
    test_case.run()?;
    Ok(())
}

#[test]
fn nested_for_loop() -> Result<(), Box<dyn std::error::Error>> {
    let test_case = read_test_case("for-loop-nested.md")?;
    test_case.run()?;
    Ok(())
}

#[ignore = "wait for it"]
#[test]
fn single_for_loop_with_where_clause() -> Result<(), Box<dyn std::error::Error>> {
    let test_case = read_test_case("for-loop-where-clause.md")?;
    test_case.run()?;
    Ok(())
}
