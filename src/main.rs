use anyhow::{bail, Result};
use clap::Parser;
use std::fmt;
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
            println!("{lp}");
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
    let (size, box_rows, box_cols) = match puzzle.len() {
        81 => (9, 3, 3), // standard sudoku
        36 => (6, 2, 3), // 6x6, this is the only one where num box rows != box cols
        16 => (4, 2, 2), // 4x4
        _ => bail!("Expected 9x9, 6x6, or 4x4 puzzle"),
    };

    // lambda to compute variable index
    let x = |row, col, value| row * size * size + col * size + value;

    let objective = vec![1; size * size * size];
    let mut b = vec![];
    let mut constraints = vec![];

    // Each cell x_rc contains one value
    for row in 0..size {
        for col in 0..size {
            let constraint: Vec<(usize, i64)> = (0..size).map(|v| (x(row, col, v), 1)).collect();
            constraints.push(constraint);
            b.push(1);
        }
    }

    // A value only appears once in each row
    for row in 0..size {
        for v in 0..size {
            let constraint: Vec<(usize, i64)> = (0..size).map(|col| (x(row, col, v), 1)).collect();
            constraints.push(constraint);
            b.push(1);
        }
    }

    // A value only appears once in each col
    for col in 0..size {
        for v in 0..size {
            let constraint: Vec<(usize, i64)> = (0..size).map(|row| (x(row, col, v), 1)).collect();
            constraints.push(constraint);
            b.push(1);
        }
    }

    // A value appears only once in each subgrid
    for subgrid in 0..size {
        let start_row = subgrid / (size / box_cols) * box_rows;
        let start_col = subgrid % (size / box_cols) * box_cols;
        for v in 0..size {
            let mut constraint = vec![];
            for r in 0..box_rows {
                let row = r + start_row;
                for c in 0..box_cols {
                    let col = c + start_col;
                    constraint.push((x(row, col, v), 1));
                }
            }
            b.push(1);
        }
    }

    Ok(LpInfo {
        objective,
        b,
        constraints,
    })
}

impl fmt::Display for LpInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Minimize")?;
        let obj = self
            .objective
            .iter()
            .enumerate()
            .map(|(i, val)| format!("{val} x{i}"))
            .collect::<Vec<String>>()
            .join(" + ");
        writeln!(f, "{obj}")?;
        writeln!(f, "Subject To")?;
        for (constraint, rhs) in self.constraints.iter().zip(&self.b) {
            writeln!(f, "{}", constraint2eqn(constraint, rhs))?;
        }
        writeln!(f, "")
    }
}

fn constraint2eqn(constraint: &Vec<(usize, i64)>, rhs: &i64) -> String {
    let eqn = constraint
        .iter()
        .map(|(index, val)| format!("{val} x{index}"))
        .collect::<Vec<String>>()
        .join(" + ");
    format!("{eqn} >= {rhs}")
}
