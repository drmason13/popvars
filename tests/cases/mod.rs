use std::path::PathBuf;

pub use self::test_utils::TestCase;
use anyhow::Context;
use erreport::path::ErrorPaths;

use parsely::result_ext::ResultExtParselyError;

mod test_utils;

pub fn read_test_case(name: &str) -> anyhow::Result<TestCase> {
    let path = if name.starts_with("tests") {
        PathBuf::from(name)
    } else {
        PathBuf::from("tests").join("cases").join(name)
    };
    let input = std::fs::read_to_string(&path).path(path)?;
    let (test_case, _) = test_utils::parse_test_case(input.as_str()).own_err()?;
    Ok(test_case)
}

pub fn run_test_cases(dir: &str) -> anyhow::Result<()> {
    let dir = PathBuf::from("tests").join("cases").join(dir);
    for entry in std::fs::read_dir(&dir)?.map(|res| res.map(|e| e.file_name())) {
        let path = entry?;
        let test_case = read_test_case(dir.join(&path).to_string_lossy().as_ref())?;
        test_case
            .run()
            .with_context(|| format!("Test case: {:?}", path))?;
    }

    Ok(())
}
