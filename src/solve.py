import gurobipy as gp
import argparse
import numpy as np
from tabulate import tabulate

parser = argparse.ArgumentParser()
parser.add_argument("lp")
args = parser.parse_args()

model = gp.read(args.lp)
model.optimize()

sz = int(round((len(model.X) - 1) ** (1 / 3)))
puzzle = np.zeros((sz, sz), dtype=int)
for i, x in enumerate(model.X[1:]):
    if x > 0.5:
        row = i // sz // sz
        col = (i // sz) % sz
        puzzle[row][col] = (i % sz) + 1

print(tabulate(puzzle, tablefmt="rounded_grid"))
