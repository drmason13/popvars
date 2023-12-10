use std::path::PathBuf;

use clap::Parser;
use popvars::Definition;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The path to a csv file containing vars to be populated
    #[arg(short, long)]
    vars: PathBuf,

    #[arg(long)]
    types: Vec<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    // read args
    /*
        popvars TEMPLATE - path to the template text file to be populated

       -d, --defs : path to a .ods spreadsheet containing vars and type definitions.
       -v, --vars : path to a .csv file containing vars
       --types : path to a .csv file containing a type (can be specified multiple times to pull in multiple types) the type name will be the filename
       -o, --out : path
    */

    // read definitions spreadsheet/csv file(s) and create definitions
    let definitions = Definition::from_csv_files(&cli.vars, &cli.types);

    println!("{definitions:?}");

    Ok(())
}
