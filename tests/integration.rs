mod cases;
use cases::{read_test_case, run_test_cases};

#[test]
fn simple() -> Result<(), Box<dyn std::error::Error>> {
    let test_case = read_test_case("simple.md")?;
    test_case.run()?;
    Ok(())
}

#[test]
fn for_loop_test_cases() -> Result<(), Box<dyn std::error::Error>> {
    Ok(run_test_cases("for-loop")?)
}

#[test]
fn if_test_cases() -> Result<(), Box<dyn std::error::Error>> {
    Ok(run_test_cases("if")?)
}

#[ignore = "includes are not implemented yet"]
#[test]
fn include_test_cases() -> Result<(), Box<dyn std::error::Error>> {
    Ok(run_test_cases("include")?)
}
