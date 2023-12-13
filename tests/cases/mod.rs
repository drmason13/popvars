use std::path::PathBuf;

pub use self::test_utils::TestCase;
use erreport::path::ErrorPaths;

use parsely::result_ext::ResultExtParselyError;

mod test_utils;

pub fn read_test_case(name: &str) -> Result<TestCase, Box<dyn std::error::Error>> {
    let path = PathBuf::from("tests").join("cases").join(name);
    let input = std::fs::read_to_string(&path).path(path)?;
    let (test_case, _) = test_utils::parse_test_case(input.as_str()).own_err()?;
    Ok(test_case)
}
