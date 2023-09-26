use anyhow::{bail, Result};
use clap::Parser;
use std::path::{Path, PathBuf};

/// A simple utility to convert standard-format Sudoku puzzles
/// to LP (for use with a binary-integer, linear-programming solver).
#[derive(Debug, Parser)]
#[clap(name = "sudoku2lp", version = "0.1.0", author = "Andrew Henshaw")]
pub struct AppArgs {
    file: PathBuf,
}

#[derive(Debug)]
struct LpInfo {
    pub objective: Vec<i64>,
    pub b: Vec<i64>,
    pub constraints: Vec<Vec<(usize, i64)>>,
}

fn main() {
    let args = AppArgs::parse();
    if let Ok(puzzle) = load(&args.file) {
        println!("{puzzle}");
        if let Ok(lp) = generate(&puzzle) {
            dbg!(lp);
        }
    }
}

/// Load and normalize the puzzle
fn load(file: &Path) -> Result<String> {
    let puzzle = std::fs::read_to_string(file)?;
    let mut puzzle = puzzle.replace(".", "0");
    puzzle.retain(|c| c.is_digit(10));
    Ok(puzzle)
}

/// Create LP from normalized puzzle.  Assumptions
/// are that the puzzle is either 4x4, 6x6, or 9x9.
fn generate(puzzle: &str) -> Result<LpInfo> {
    let (size, _box_rows) = match puzzle.len() {
        81 => (9, 3), // standard sudoku
        36 => (6, 2), // 6x6, this is the only one where num box rows != box cols
        16 => (4, 2), // 4x4
        _ => bail!("Expected 9x9, 6x6, or 4x4 puzzle"),
    };

    let objective = vec![1; size * size * size];
    let b = vec![];
    let constraints = vec![vec![]];

    Ok(LpInfo {
        objective,
        b,
        constraints,
    })
}
