use anyhow::{bail, Result};
use clap::Parser;
use good_lp::solvers::lp_solvers::{GurobiSolver, LpSolver, Model};
use good_lp::{variable, Expression, ProblemVariables, SolverModel, Variable};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A simple utility to convert standard-format Sudoku puzzles
/// to LP (for use with a binary-integer, linear-programming solver).
#[derive(Debug, Parser)]
#[clap(name = "sudoku2lp", version = "0.1.0", author = "Andrew Henshaw")]
pub struct AppArgs {
    in_file: PathBuf,
    out_file: Option<PathBuf>,
    #[arg(short, long, help = "Solve puzzle (if solver available)")]
    solve: bool,
}

fn main() {
    let args = AppArgs::parse();

    // if out_file not provided, use in_file base + ".lp"
    let mut out_file = args.out_file.unwrap_or(args.in_file.clone());
    out_file.set_extension("lp");

    if let Ok(puzzle) = load(&args.in_file) {
        if let Ok(model) = generate(&puzzle) {
            if args.solve {
                match model.solve() {
                    Ok(_solution) => println!("solved"),
                    Err(e) => eprintln!("{e}"),
                }
            }
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
fn generate(puzzle: &str) -> Result<Model<GurobiSolver>> {
    let (size, box_rows, box_cols) = match puzzle.len() {
        81 => (9, 3, 3), // standard sudoku
        36 => (6, 2, 3), // 6x6, this is the only one where num box rows != box cols
        16 => (4, 2, 2), // 4x4
        _ => bail!("Expected 9x9, 6x6, or 4x4 puzzle"),
    };

    // create all the variables and store
    // them in a HashMap for later reference
    type Idx = (usize, usize, usize);
    let mut problem = ProblemVariables::new();
    let mut x: HashMap<Idx, Variable> = HashMap::new();
    for row in 1..=size {
        for col in 1..=size {
            for num in 1..=size {
                let idx = (row, col, num);
                let name = format!("x_{row}_{col}_{num}");
                x.insert(idx, problem.add(variable().binary().name(name)));
            }
        }
    }

    let gurobi =
        GurobiSolver::new().command_name("/opt/gurobi1001/linux64/bin/gurobi_cl".to_string());
    let solver = LpSolver(gurobi);
    let mut model = problem.minimise(x.get(&(1, 1, 1)).unwrap()).using(solver);

    // Each cell x[r,c] contains one value
    for row in 1..=size {
        for col in 1..=size {
            let mut eqn: good_lp::Expression = Expression::with_capacity(size);
            for num in 1..=size {
                eqn += x.get(&(row, col, num)).unwrap();
            }
            model.add_constraint(eqn.eq(1));
        }
    }

    // A value only appears once in each row
    for row in 1..size {
        for num in 1..size {
            let mut eqn: good_lp::Expression = Expression::with_capacity(size);
            for col in 1..=size {
                eqn += x.get(&(row, col, num)).unwrap();
            }
            model.add_constraint(eqn.eq(1));
        }
    }

    // A value only appears once in each col
    for col in 1..=size {
        for num in 1..=size {
            let mut eqn: good_lp::Expression = Expression::with_capacity(size);
            for row in 1..=size {
                eqn += x.get(&(row, col, num)).unwrap();
            }
            model.add_constraint(eqn.eq(1));
        }
    }

    // A value appears only once in each subgrid
    for subgrid in 0..size {
        let start_row = subgrid / (size / box_cols) * box_rows;
        let start_col = subgrid % (size / box_cols) * box_cols;
        for num in 1..=size {
            let mut eqn: good_lp::Expression = Expression::with_capacity(size);
            for r in 0..box_rows {
                let row = r + start_row + 1;
                for c in 0..box_cols {
                    let col = c + start_col + 1;
                    eqn += x.get(&(row, col, num)).unwrap();
                }
            }
            model.add_constraint(eqn.eq(1));
        }
    }

    // The original clues from the puzzle
    let mut eqn: good_lp::Expression = Expression::with_capacity(size * size);
    let mut count = 0;
    for (i, c) in puzzle.chars().enumerate().filter(|(_, c)| *c != '0') {
        // We know this unwrap can't fail
        let val = c.to_digit(10).unwrap() as usize;
        let row = i / size + 1;
        let col = i % size + 1;
        eqn += x.get(&(row, col, val)).unwrap();
        count += 1;
    }
    model.add_constraint(eqn.geq(count));

    Ok(model)
}
