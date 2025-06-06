"""
This module provides an implementation of an Optimal Bi-State Subset Selection Problem,
which involves selecting a subset of items that minimize a weighted sum of values subject to
multiple state constraints. This problem can be viewed as a generalization of the Knapsack Problem,
and it has applications in various domains such as resource allocation, portfolio optimization,
and feature selection.
The module uses the CBC solver from OR-Tools via pywraplp to find an optimal solution.
"""

# Note:
# The speed of the CBC solver can be improved by limiting the variable entries for the states to
# only those with a positive value. HiGHS will do this while the model is being created but for CBC
# they need to be validated prior to inserting them into the model.
#
# For this particular problem and the sample datasets the improvement is around 20% but I opt to
# keep the modelling more general and inline with the Rust/HiGHS solution for comparison and ease
# of understanding and testing.

import contextlib
import io
import os
import re
import sys

from collections import defaultdict

# Suppress stdout during import
with contextlib.redirect_stdout(io.StringIO()):
    from ortools.linear_solver import pywraplp


def subset_solver(items, item_reqs, weights, state_1_values, state_2_values):
    """
    - items is a list of items to choose from.
    - item_reqs is a list of item requirements, where item_reqs[i] is the parent item of items[i].
    - weights is a list of item weights, where weights[i] is the weight of items[i].
    - state_1_values and state_2_values are lists of item values in each state, where
      state_1_values[i] and state_2_values[i] are the values of items[i] in states 1 and 2,
      respectively.
    """

    # parent -> child relation tree for item selection requirements.
    item_req_tree = defaultdict(list)
    for i, item_req in enumerate(item_reqs):
        if item_req == 0:
            item_req_tree[items[0]].append(items[i])
        else:
            item_req_tree[item_req].append(items[i])

    # Use the COIN Branch and Cut solver
    solver = pywraplp.Solver("subset_selection", pywraplp.Solver.CBC_MIXED_INTEGER_PROGRAMMING)

    # Variables to flag selected items and indicate the state of the selected item.
    # solver.BoolVar() returns an anonymous variable index.
    # solver allows assignment and retrieval by name.
    item_flags = {}
    state_1_flags = {}
    state_2_flags = {}
    for item in items:
        item_flags[item] = solver.BoolVar(f"key_{item}")
        state_1_flags[item] = solver.BoolVar(f"state_1_flag_{item}")
        state_2_flags[item] = solver.BoolVar(f"state_2_flag_{item}")

    # The item requirements constraints.
    # Transitive; ensures children must have all ancestors back to root.
    for parent, children in item_req_tree.items():
        for child in children:
            if parent == items[0] or parent == 0:
                continue
            solver.Add(item_flags[child] - item_flags[parent] <= 0)

    for item in items:
        if item == items[0]:
            continue
        # Item selection constraint: one state on flagged items, no state otherwise.
        solver.Add(state_1_flags[item] + state_2_flags[item] - item_flags[item] == 0)

    state_1_sum_constraint = solver.Constraint(9999, solver.infinity(), "state_1_lb")
    state_2_sum_constraint = solver.Constraint(9999, solver.infinity(), "state_2_lb")
    for i, item in enumerate(items):
        if i == 0:
            continue
        # Item selected as state 1 values
        state_1_sum_constraint.SetCoefficient(state_1_flags[item], state_1_values[i])
        # Item selected as state 2 values
        state_2_sum_constraint.SetCoefficient(state_2_flags[item], state_2_values[i])

    # The objective: select "items" that minimize the weight subject to value sum lb constraints.
    objective = solver.Objective()
    for i, item in enumerate(items):
        if i == 0:
            continue
        # Item selection cost.
        objective.SetCoefficient(item_flags[item], weights[i])
    objective.SetMinimization()

    # Force root variables to be used so they get included in MPS export
    root = items[0]
    objective.SetCoefficient(item_flags[root], 1e-9)
    state_1_sum_constraint.SetCoefficient(state_1_flags[root], 1e-9)
    state_2_sum_constraint.SetCoefficient(state_2_flags[root], 1e-9)

    return solver


def subset_selection(
    items,
    item_reqs,
    weights,
    state_1_values,
    state_2_values,
    state_1_sum_values_lb,
    state_2_sum_values_lb,
):
    """
    - items is a list of items to choose from.
    - item_reqs is a list of item requirements, where item_reqs[i] is the parent item of items[i].
    - weights is a list of item weights, where weights[i] is the weight of items[i].
    - state_1_values and state_2_values are lists of item values in each state, where
      state_1_values[i] and state_2_values[i] are the values of items[i] in states 1 and 2,
      respectively.
    - state_1_sum_values_lb and state_2_sum_values_lb are lower bounds on the sum of the values of
      the selected items in each state.
    """
    solver = subset_solver(items, item_reqs, weights, state_1_values, state_2_values)

    lb1 = 0.0 if state_1_sum_values_lb is None else float(state_1_sum_values_lb)
    lb2 = 0.0 if state_2_sum_values_lb is None else float(state_2_sum_values_lb)

    state_1_constraint = solver.LookupConstraint("state_1_lb")
    state_2_constraint = solver.LookupConstraint("state_2_lb")
    state_1_constraint.SetBounds(lb1, solver.infinity())
    state_2_constraint.SetBounds(lb2, solver.infinity())

    result_status = solver.Solve()

    if result_status == pywraplp.Solver.OPTIMAL:
        return extract_solution(solver, items, weights, state_1_values, state_2_values)
    return (None, None, None, None, None)


def subset_selection_par(
    items,
    item_reqs,
    weights,
    state_1_values,
    state_2_values,
    state_1_values_sum_ub,
    state_2_values_sum_lb,
):
    """Optimize all state pairs in parallel."""
    solutions = []
    solver = subset_solver(items, item_reqs, weights, state_1_values, state_2_values)

    state_1_constraint = solver.LookupConstraint("state_1_lb")
    state_2_constraint = solver.LookupConstraint("state_2_lb")

    state_1_values_sum_ub = 0.0 if state_1_values_sum_ub is None else float(state_1_values_sum_ub)
    state_2_values_sum_lb = 0.0 if state_2_values_sum_lb is None else float(state_2_values_sum_lb)

    state_2_constraint.SetBounds(state_2_values_sum_lb, solver.infinity())
    state_1_values_sum_lb = 0.0
    while state_1_values_sum_lb <= state_1_values_sum_ub:
        state_1_constraint.SetBounds(state_1_values_sum_lb, solver.infinity())
        result_status = solver.Solve()

        if result_status == pywraplp.Solver.OPTIMAL:
            solution = extract_solution(solver, items, weights, state_1_values, state_2_values)
            solutions.append(solution)
            state_1_values_sum_lb = solution[1] + 1
        else:
            break

    return solutions


def extract_solution(solver, items, weights, state_1_values, state_2_values):
    """
    # Extract the solution.
    # Sum the total solution weight and sums used in the state constraints
    # and collect the selected items and their states.
    """
    total_weight = 0
    state_1_sum = 0
    state_2_sum = 0
    selected_keys = []
    selected_key_states = []

    for i, item in enumerate(items):
        if i == 0:
            continue
        item_flag = solver.LookupVariable(f"key_{item}")
        if item_flag.solution_value() == 1:
            selected_keys += [item]
            total_weight += weights[i]
            if solver.LookupVariable(f"state_1_flag_{item}").solution_value() == 1:
                selected_key_states += [1]
                state_1_sum += state_1_values[i]
            elif solver.LookupVariable(f"state_2_flag_{item}").solution_value() == 1:
                selected_key_states += [2]
                state_2_sum += state_2_values[i]

    # Since the results of this model are only being used to validate the model and cross-check
    # the results of the HiGHS results from the Rust optimization we do not need the selected_keys
    # or selected_key_states since different optimizers and different ordering of the input will
    # readily yield different item groups that meet the constraints.
    # return (total_weight, state_1_sum, state_2_sum, selected_keys, selected_key_states)
    return (total_weight, state_1_sum, state_2_sum, [], [])


def write_subset_selection_mps(items, item_reqs, weights, state_1_values, state_2_values):
    """Instantiate model and write to mps file 'subset_select(N).mps'"""
    solver = subset_solver(items, item_reqs, weights, state_1_values, state_2_values)
    state_1_constraint = solver.LookupConstraint("state_1_lb")
    state_2_constraint = solver.LookupConstraint("state_2_lb")
    state_1_constraint.SetBounds(9999, solver.infinity())
    state_2_constraint.SetBounds(9999, solver.infinity())
    model_text = solver.ExportModelAsMpsFormat(fixed_format=True, obfuscate=True)
    write_to_file(model_text, "subset_select.mps")
    sys.exit("Model written.")


def write_to_file(data, filename):
    """Write mps file with incremental (n) suffix."""
    i = 1
    name, ext = os.path.splitext(filename)
    while os.path.exists(filename):
        name = re.sub(r"\(\d+\)$", "", name)
        filename = f"{name}({i}){ext}"
        i += 1
    with open(filename, "w", encoding="UTF-8") as file:
        file.write(data)
