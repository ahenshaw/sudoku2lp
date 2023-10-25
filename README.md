# Sudoku-to-LP
Takes a sudoku puzzle in standard text format and generates an LP version.

This is written in Rust, but there is also a small Python program that
reads the generated LP file into a Gurobi model, solves it, and then prints
the solution.

