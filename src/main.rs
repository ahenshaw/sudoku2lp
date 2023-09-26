use anyhow::Result;
use clap::Parser;
use std::path::{Path, PathBuf};

/// A simple utility to convert standard-format Sudoku puzzles
/// to LP (for use with a binary-integer, linear-programming solver).
#[derive(Debug, Parser)]
#[clap(name = "sudoku2lp", version = "0.1.0", author = "Andrew Henshaw")]
pub struct AppArgs {
    file: PathBuf,
}

fn main() {
    let args = AppArgs::parse();
    if let Ok(puzzle) = load(&args.file) {
        println!("{puzzle}");
    }
}

/// Load and normalize the puzzle
fn load(file: &Path) -> Result<String> {
    let data = std::fs::read_to_string(file)?;
    let mut data = data.replace(".", "0");
    data.retain(|c| !c.is_ascii_whitespace());
    Ok(data)
}
