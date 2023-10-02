use anyhow::{bail, Result};
use clap::Parser;
use std::fmt;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

/// A simple utility to convert standard-format Sudoku puzzles
/// to LP (for use with a binary-integer, linear-programming solver).
#[derive(Debug, Parser)]
#[clap(name = "sudoku2lp", version = "0.1.0", author = "Andrew Henshaw")]
pub struct AppArgs {
    in_file: PathBuf,
    out_file: Option<PathBuf>,
}

struct LpInfo {
    pub objective: Vec<i64>,
    pub b: Vec<Equality>,
    pub constraints: Vec<Vec<(usize, i64)>>,
}

fn main() {
    let args = AppArgs::parse();

    // if out_file not provided, use in_file base + ".lp"
    let mut out_file = args.out_file.unwrap_or(args.in_file.clone());
    out_file.set_extension("lp");

    if let Ok(puzzle) = load(&args.in_file) {
        if let Ok(lp) = generate(&puzzle) {
            let mut output = File::create(out_file).expect("File I/O");
            write!(output, "{}", lp).expect("File I/O");
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
            b.push(Equality::EQ(1));
        }
    }

    // A value only appears once in each row
    for row in 0..size {
        for v in 0..size {
            let constraint: Vec<(usize, i64)> = (0..size).map(|col| (x(row, col, v), 1)).collect();
            constraints.push(constraint);
            b.push(Equality::EQ(1));
        }
    }

    // A value only appears once in each col
    for col in 0..size {
        for v in 0..size {
            let constraint: Vec<(usize, i64)> = (0..size).map(|row| (x(row, col, v), 1)).collect();
            constraints.push(constraint);
            b.push(Equality::EQ(1));
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
            constraints.push(constraint);
            b.push(Equality::EQ(1));
        }
    }

    // The original clues from the puzzle
    let constraint: Vec<(usize, i64)> = puzzle
        .chars()
        .enumerate()
        .filter(|(_, c)| *c != '0')
        .map(|(i, c)| {
            // We know this unwrap can't fail
            let val = c.to_digit(10).unwrap() as usize;
            let row = i / size;
            let col = i % size;
            (x(row, col, val - 1), 1)
        })
        .collect();

    b.push(Equality::GE(constraint.len() as i64));
    constraints.push(constraint);

    Ok(LpInfo {
        objective,
        b,
        constraints,
    })
}

impl fmt::Display for LpInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let vars = (0..self.objective.len())
            .map(|i| format!("x{i}"))
            .collect::<Vec<String>>();

        writeln!(f, "Minimize\n0")?;
        writeln!(f, "Subject To")?;
        for (constraint, rhs) in self.constraints.iter().zip(&self.b) {
            writeln!(f, "{}", constraint2eqn(constraint, rhs))?;
        }
        writeln!(f, "Binary")?;
        writeln!(f, "{}", vars.join(" "))?;
        writeln!(f, "End")
    }
}

fn constraint2eqn(constraint: &Vec<(usize, i64)>, rhs: &Equality) -> String {
    let eqn = constraint
        .iter()
        .map(|(index, val)| format!("{val} x{index}"))
        .collect::<Vec<String>>()
        .join(" + ");
    format!("{eqn} {rhs}")
}

enum Equality {
    EQ(i64),
    GE(i64),
}

impl fmt::Display for Equality {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Equality::EQ(value) => format!(" = {value}"),
            Equality::GE(value) => format!(">= {value}"),
        };
        write!(f, "{s}")
    }
}
