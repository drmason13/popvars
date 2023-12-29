use std::path::PathBuf;

use clap::Parser;
use popvars::Definition;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The path to a csv file containing vars to be populated
    #[arg(short, long)]
    vars: PathBuf,

    /// The path to the template file to render
    #[arg(short, long)]
    template: PathBuf,

    /// path to a .csv file containing a type (can be specified multiple times to pull in multiple types) the type name will be the filename
    #[arg(long)]
    types: Vec<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let template = std::fs::read_to_string(&cli.template)?;

    let definitions = Definition::from_csv_files(&cli.vars, &cli.types)?;

    let popped = popvars::pop(&template, definitions)?;

    #[cfg(windows)]
    println!("{}", popped.join("\r\n"));

    #[cfg(unix)]
    println!("{}", popped.join("\n"));

    Ok(())
}
