"""
This module is simply a front-end interface to the optimize_subset_selection.
"""

import argparse
from collections import namedtuple
import copy
import itertools
import json
from math import ceil
import multiprocessing as mp
import random
import sys

from houseinfo import get_region_buildings
from optimize_key_selection import subset_selection, subset_selection_par, write_subset_selection_mps

OptimizerResult = namedtuple("Solution", 'cost storage lodging items states')
Solution = namedtuple("Solution", 'lodging storage cost items states')
RegionInfo = namedtuple("RegionInfo",
                        'buildings, items, item_reqs, weights, state_1_values, state_2_values')


def are_solutions_equal(s_1, s_2):
    return (s_1.cost == s_2.cost) and (s_1.storage == s_2.storage) and (
        s_1.lodging == s_2.lodging) and (s_1.items == s_2.items) and (s_1.states == s_2.states)


def dominates(s1: Solution, s2: Solution):
    return s1.cost <= s2.cost and s1.storage >= s2.storage and s1.lodging >= s2.lodging and not (
        s1.cost == s2.cost and s1.storage == s2.storage and s1.lodging == s2.lodging)


def elegant_pair(x, y):
    if x != max(x, y):
        return pow(y, 2) + x
    return pow(x, 2) + x + y


def get_region_info(region_name):
    return RegionInfo(*get_region_buildings(region_name))


def get_exact_region_solutions(region_name):
    region_name = region_name.replace(" ", "_")
    with open(f"./data/housecraft/{region_name}.json", encoding="UTF-8") as file:
        exact_solutions = json.load(file)
    return exact_solutions


def write_model(region: RegionInfo, _lodging, _storage):
    write_subset_selection_mps(region.items, region.item_reqs, region.weights,
                               region.state_1_values, region.state_2_values)


def optimize(region: RegionInfo, lodging, storage):
    s = subset_selection(region.items, region.item_reqs, region.weights, region.state_1_values,
                         region.state_2_values, storage, lodging)
    result = OptimizerResult(*s)
    return Solution(result.lodging, result.storage, result.cost, result.items, result.states)


def optimize_all(args, region_info: RegionInfo):
    max_storage = sum(region_info.state_1_values)
    max_lodging = sum(region_info.state_2_values)
    print(
        f"Generating for {args.region} with max lodging and storage of ({max_lodging}, {max_storage})"
    )

    if args.jobs > 0:
        solutions = optimize_all_state_pairs_par(args, region_info)
    else:
        solutions = optimize_all_state_pairs(region_info)

    solutions = sorted(solutions, key=lambda os: (os.lodging, os.storage, os.cost))

    solutions = remove_duplicates(solutions)
    solutions = retain_top_by_lodging_storage(region_info, solutions)
    solutions = retain_dominating(solutions)
    solutions = sorted(solutions, key=lambda os: (os.lodging, os.storage, os.cost))

    if args.validate:
        validate_solutions(args, solutions)
    else:
        solutions = [json.dumps(s._asdict()) for s in solutions]
        if not args.quiet:
            for solution in solutions:
                print(solution)
        print(f"Generated: {len(solutions)} dominating lodging/storage chains.")


def optimize_all_state_pairs(region_info: RegionInfo):
    solutions = []
    max_storage = sum(region_info.state_1_values)
    max_lodging = sum(region_info.state_2_values)
    for lodging in range(max_lodging + 1):
        for storage in range(max_storage + 1):
            solution = optimize(region_info, lodging, storage)
            if solution.items is not None:
                solutions.append(solution)
    print(f"Captured count {len(solutions)}")
    return solutions


def optimize_all_state_pairs_par(args, region_info: RegionInfo):
    worker_args = optimizer_par_worker_args(args, region_info)
    solutions = []
    with mp.Pool(args.jobs) as pool:
        worker_solutions = pool.map(optimize_all_state_pairs_par_worker, worker_args)
    for worker_solution in worker_solutions:
        solutions += worker_solution

    print(f"Captured count {len(solutions)}")
    return solutions


def optimize_all_state_pairs_par_worker(worker_args):
    region = worker_args["region_info"]
    subset_solutions = subset_selection_par(region.items, region.item_reqs, region.weights,
                                            region.state_1_values, region.state_2_values,
                                            worker_args["params"])
    solutions = []
    for solution in subset_solutions:
        result = OptimizerResult(*solution)
        s = Solution(result.lodging, result.storage, result.cost, result.items, result.states)
        solutions.append(s)
    return solutions


def optimizer_par_worker_args(args, region_info):
    max_storage = sum(region_info.state_1_values)
    max_lodging = sum(region_info.state_2_values)
    lodging_storage_pairs = list(itertools.product(range(max_storage + 1), range(max_lodging + 1)))
    random.shuffle(lodging_storage_pairs)
    chunked_pairs = split_list_into_n_parts(lodging_storage_pairs, args.jobs)

    worker_args = []
    for chunk in chunked_pairs:
        worker_args.append({
            "region_info": copy.deepcopy(region_info),
            "params": copy.deepcopy(chunk)
        })

    return worker_args


def remove_duplicates(solutions_list):
    unique_solutions = []
    for solution in solutions_list:
        if not any(
                are_solutions_equal(solution, unique_solution)
                for unique_solution in unique_solutions):
            unique_solutions.append(solution)
    return unique_solutions


def retain_dominating(solutions: list[Solution]):
    j = 0
    n = len(solutions)
    for i in range(n):
        if all(not dominates(solutions[a], solutions[i])
               for a in list(range(0, j)) + list(range(i + 1, n))):
            solutions[i], solutions[j] = solutions[j], solutions[i]
            j += 1
    return solutions[:j]


def retain_top_by_lodging_storage(region_info, solutions):
    dim_x = sum(region_info.state_1_values)
    dim_y = sum(region_info.state_2_values)
    dim_len = pow(max(dim_x, dim_y) + 1, 2)

    seen_cost = [9999] * dim_len
    kept_idices = [None] * dim_len
    kept_solutions = []
    for solution in solutions:
        key_index = elegant_pair(solution.lodging, solution.storage)
        if seen_cost[key_index] == 9999:
            # insert
            seen_cost[key_index] = solution.cost
            kept_idices[key_index] = len(kept_solutions)
            kept_solutions.append(solution)
        elif seen_cost[key_index] < solution.cost:
            # update
            seen_cost[key_index] = solution.cost
            kept_index = kept_idices[key_index]
            kept_solutions[kept_index] = solution
    return kept_solutions


def split_list_into_n_parts(lst, n):
    size = ceil(len(lst) / n)
    return list(map(lambda x: lst[x * size:x * size + size], list(range(n))))


def validate_solutions(args, optimized_solutions):
    print("Validating...", end=" ")
    exact_solutions = get_exact_region_solutions(args.region)
    exact_solutions = sorted(exact_solutions,
                             key=lambda es: (es["lodging"], es["storage"], es["cost"]))

    for os, es in zip(optimized_solutions, exact_solutions):
        if os.cost != es["cost"] or os.lodging != es["lodging"] or os.storage != es["storage"]:
            print("[failed]")
            print(f"  os: {os}")
            print(f"  es: {es}")
    passed = len(optimized_solutions) == len(exact_solutions)
    print(
        f"{len(optimized_solutions)} optimized of {len(exact_solutions)} exact solutions: {passed}."
    )


def main(args):
    # if args.validate and args.region in ["Calpheon City", "Valencia City", "Heidel"]:
    #     sys.exit("Exact results are unavailable to validate the optimizer against.")

    region_info = get_region_info(args.region)
    if args.write:
        write_model(region_info, args.lodging, args.storage)
    elif args.all:
        optimize_all(args, region_info)
    else:
        solution = optimize(region_info, args.lodging, args.storage)
        print(f"cost: {solution.cost}, lodging: {solution.lodging}, storage: {solution.storage}\n"
              f"items: {solution.items}\n"
              f"states: {solution.states}\n")


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument("-r", "--region", help="warehouse region")
    parser.add_argument("-l", "--lodging", help="desired lodging", type=int)
    parser.add_argument("-s", "--storage", help="desired storage", type=int)
    parser.add_argument("-A",
                        "--all",
                        help="generate all dominant (ignores --lodging/--storage)",
                        action=argparse.BooleanOptionalAction)
    parser.add_argument("-q",
                        "--quiet",
                        help="suppress listing",
                        action=argparse.BooleanOptionalAction)
    parser.add_argument("-j", "--jobs", help="number of worker jobs to use", type=int, default=0)
    parser.add_argument("-V",
                        "--validate",
                        help="validate against exact (only has meaning with --All)",
                        action=argparse.BooleanOptionalAction)
    parser.add_argument("-W",
                        "--write",
                        help="write model to mps file (ignores all other arguments)",
                        action=argparse.BooleanOptionalAction)
    main(parser.parse_args())
