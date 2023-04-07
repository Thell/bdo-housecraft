"""
This module provides an implementation of an Optimal Bi-State Subset Selection Problem,
which involves selecting a subset of items that minimize a weighted sum of values subject to
multiple state constraints. This problem can be viewed as a generalization of the Knapsack Problem,
and it has applications in various domains such as resource allocation, portfolio optimization,
and feature selection.
The module uses the CBC solver from OR-Tools via pywraplp to find an optimal solution.
"""

from collections import defaultdict
from ortools.linear_solver import pywraplp


def optimize_subset_selection(items, item_reqs, weights, state_1_values, state_2_values,
                              state_1_sum_values_lb, state_2_sum_values_lb):
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

    # Use the COIN Branch and Cut solver
    solver = pywraplp.Solver('subset_selection', pywraplp.Solver.CBC_MIXED_INTEGER_PROGRAMMING)

    # The decision variables.
    # Variables to flag selected items and indicate the state of the item.
    item_flags = {}
    state_1_flags = {}
    state_2_flags = {}
    for item in items:
        item_flags[item] = solver.BoolVar(f"key_{item}")
        state_1_flags[item] = solver.BoolVar(f"state_1_indicator_{item}")
        state_2_flags[item] = solver.BoolVar(f"state_2_indicator_{item}")

    # The objective: select "items" that minimize the weight subject to value sum lb constraints.
    objective = solver.Objective()
    for i, item_flag in enumerate(item_flags.values()):
        objective.SetCoefficient(item_flag, weights[i])
    objective.SetMinimization()

    # The item requirements constraints.
    # This constraint is transitive to all ancestors of item such that every item selected requires
    # all ancestor items to be selected where item_req => item is a parent => child relation.
    tree = defaultdict(list)
    for i, item_req in enumerate(item_reqs):
        if item_req == 0:
            tree[items[0]].append(items[i])
        else:
            tree[item_req].append(items[i])
    for parent, children in tree.items():
        for child in children:
            solver.Add(item_flags[child] <= item_flags[parent])

    # The state constraints.
    state_1_constraint = solver.Constraint(state_1_sum_values_lb, solver.infinity())
    state_2_constraint = solver.Constraint(state_2_sum_values_lb, solver.infinity())
    for i, (item, item_flag) in enumerate(item_flags.items()):
        if i == 0:  # skip the root item
            continue

        # The "state_1_value" constraints.
        state_1_constraint.SetCoefficient(state_1_flags[item], state_1_values[i])
        solver.Add(state_1_flags[item] <= item_flag)
        solver.Add(state_1_flags[item] <= state_1_flags[item] * state_1_values[i])

        # The "state_2_value" constraints.
        state_2_constraint.SetCoefficient(state_2_flags[item], state_2_values[i])
        solver.Add(state_2_flags[item] <= item_flag)
        solver.Add(state_2_flags[item] <= state_2_flags[item] * state_2_values[i])

        # The state flag "mutual exclusivity" constraint.
        solver.Add(state_1_flags[item] + state_2_flags[item] <= 1)
        # The mandatory state flag constraint on flagged items.
        solver.Add(item_flag <= state_1_flags[item] + state_2_flags[item])

    # Solve the problem.
    result_status = solver.Solve()

    # Extract the solution.
    if result_status == pywraplp.Solver.OPTIMAL:
        # Sum the total solution weight and sums used in the state constraints
        # and collect the selected items and their states.
        total_weight = 0
        state_1_sum = 0
        state_2_sum = 0
        selected_keys = []
        selected_key_states = []

        for i, (item, item_flag) in enumerate(item_flags.items()):
            if i == 0:
                continue
            if item_flag.solution_value() == 1:
                selected_keys += [item]
                total_weight += weights[i]
                if state_1_flags[item].solution_value() == 1:
                    selected_key_states += [1]
                    state_1_sum += state_1_values[i]
                elif state_2_flags[item].solution_value() == 1:
                    selected_key_states += [2]
                    state_2_sum += state_2_values[i]

        return (total_weight, state_1_sum, state_2_sum, selected_keys, selected_key_states)
    return (None, None, None, None, None)
