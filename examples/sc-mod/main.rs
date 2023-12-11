use std::path::PathBuf;

use popvars::Definition;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let definitions = Definition::from_csv_files(
        &PathBuf::from("national morale.csv"),
        &[
            PathBuf::from("country.csv"),
            PathBuf::from("team.csv"),
            PathBuf::from("city.csv"),
        ],
    )?;

    let input = std::fs::read_to_string("national morale.txt")?;

    let popped = popvars::pop(&input, definitions)?;

    println!("{}", popped.join("\n"));

    Ok(())
}
