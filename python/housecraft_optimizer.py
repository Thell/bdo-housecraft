"""
This module is simply a front-end interface to the optimize_subset_selection.
"""

import argparse
from collections import namedtuple
import copy
from datetime import datetime
import json
import multiprocessing as mp
import os.path

from houseinfo import get_region_buildings
from optimize_key_selection import subset_selection, subset_selection_par, write_subset_selection_mps

OptimizerResult = namedtuple("Solution", "cost storage lodging items states")
Solution = namedtuple("Solution", "lodging storage cost items states")
RegionInfo = namedtuple(
    "RegionInfo", "buildings, items, item_reqs, weights, state_1_values, state_2_values"
)

# pylint: disable=invalid-name


def dominates(s1: Solution, s2: Solution):
    """Returns true if the first solution dominates the second solution."""
    return s1.cost <= s2.cost and s1.storage >= s2.storage and s1.lodging >= s2.lodging


# pylint: enable=invalid-name


def have_validation_files(region_name):
    """returns true if validation files exist"""
    region_name = region_name.replace(" ", "_")
    if region_name in ["Altinova", "Heidel", "Valencia_City", "Calpheon_City"]:
        print(f"./data/housecraft/validation/HiGHS/{region_name}.json")
        return os.path.exists(f"./data/housecraft/validation/HiGHS/{region_name}.json")
    return os.path.exists(
        f"./data/housecraft/validation/highs/{region_name}.json"
    ) and os.path.exists(f"./data/housecraft/validation/popjumppush/{region_name}.json")


def get_highs_solutions(region_name):
    """read the HiGHS solutions json"""
    region_name = region_name.replace(" ", "_")
    with open(f"./data/housecraft/validation/HiGHS/{region_name}.json", encoding="UTF-8") as file:
        solutions = json.load(file)
    return solutions


def get_popjumppush_solutions(region_name):
    """read the popjumppush exact solutions"""
    region_name = region_name.replace(" ", "_")
    with open(
        f"./data/housecraft/validation/popjumppush/{region_name}.json", encoding="UTF-8"
    ) as file:
        exact_solutions = json.load(file)
    return exact_solutions


def get_region_info(region_name):
    """return the region's buildings"""
    return RegionInfo(*get_region_buildings(region_name))


def optimize(region: RegionInfo, lodging, storage):
    """returns the optimal solution for the given lodging, storage combination"""
    solution = subset_selection(
        region.items,
        region.item_reqs,
        region.weights,
        region.state_1_values,
        region.state_2_values,
        storage,
        lodging,
    )
    result = OptimizerResult(*solution)
    return Solution(result.lodging, result.storage, result.cost, result.items, result.states)


def optimize_all(args, region_info: RegionInfo):
    """return all optimal solutions for all lodging, storage pairs"""
    max_storage = sum(region_info.state_1_values)
    max_lodging = sum(region_info.state_2_values)
    time_log(f"Generating for {args.region} with max lodging and storage of \
             ({max_lodging}, {max_storage})")

    if args.jobs > 0:
        solutions = optimize_all_par(args, region_info)
    else:
        solutions = optimize_all_state_pairs(region_info)

    time_log("post-processing... dominating")
    solutions = retain_dominating(solutions)

    if args.validate:
        time_log(f"Retained count {len(solutions)}")
        validate_solutions(args, solutions)
    else:
        solutions = [json.dumps(s._asdict()) for s in solutions]
        if not args.quiet:
            for solution in solutions:
                print(solution)
        time_log(f"Generated: {len(solutions)} dominating lodging/storage chains.")


def optimize_all_state_pairs(region_info: RegionInfo):
    """optimize all lodging, storage pairs for the region"""
    solutions = []
    max_storage = sum(region_info.state_1_values)
    max_lodging = sum(region_info.state_2_values)

    for lodging in range(max_lodging + 1):
        storage = 0
        while storage <= max_storage + 1:
            solution = optimize(region_info, lodging, storage)
            if solution.cost is not None:
                storage = solution.storage + 1
                solutions.append(solution)
            else:
                break
    time_log(f"Captured count {len(solutions)}")
    return solutions


def optimize_all_par(args, region_info: RegionInfo):
    """optimize all lodging, storage pairs for the region using multiple workers"""
    # Use a single thread per lodging axis
    worker_args = optimizer_par_worker_args(args, region_info)
    solutions = []
    num_workers = min(len(worker_args), mp.cpu_count())
    with mp.Pool(num_workers) as pool:
        worker_solutions = pool.map(optimize_all_par_worker, worker_args)
    for worker_solution in worker_solutions:
        solutions += worker_solution

    time_log(f"Captured count {len(solutions)}")
    return solutions


def optimize_all_par_worker(worker_args):
    """optimize the lodging, storage pairs given in worker_args"""
    region = worker_args["region_info"]
    subset_solutions = subset_selection_par(
        region.items,
        region.item_reqs,
        region.weights,
        region.state_1_values,
        region.state_2_values,
        *worker_args["params"],
    )
    solutions = []
    for solution in subset_solutions:
        result = OptimizerResult(*solution)
        solution = Solution(result.lodging, result.storage, result.cost, result.items, result.states)
        solutions.append(solution)
    return solutions


def optimizer_par_worker_args(_args, region_info):
    """return a list of arguments, region and storage, lodging pairs, for each worker."""
    state_1_sum_ub = sum(region_info.state_1_values) + 1
    state_2_sum_ub = sum(region_info.state_2_values) + 1
    return [
        {"region_info": copy.deepcopy(region_info), "params": (state_1_sum_ub, lb)}
        for lb in range(state_2_sum_ub)
    ]


def retain_dominating(solutions: list[Solution]):
    """returns the dominating solutions"""
    n = len(solutions)
    dominated_indices = set()
    for i in range(n):
        if i not in dominated_indices:
            for j in range(i + 1, n):
                if dominates(solutions[i], solutions[j]):
                    dominated_indices.add(j)
                elif dominates(solutions[j], solutions[i]):
                    dominated_indices.add(i)
                    break
    result = [solutions[i] for i in range(n) if i not in dominated_indices]
    return result


def time_log(msg):
    """prints output with a timestamp"""
    print(f"{datetime.now().isoformat(sep=' ', timespec='milliseconds')}: {msg}")


def validate_solutions(args, cbc_solutions):
    """Validate optimizer solutions.
    Passing conditions:
      - CBC == popjumppush == HiGHS for all regions < Heidel
      - CBC == HiGHS for all regions >= Heidel
    """
    cbc = dict.fromkeys([tuple(s[0:3]) for s in cbc_solutions])
    highs = dict.fromkeys([tuple(s.values())[0:3] for s in get_highs_solutions(args.region)])

    time_log(f"Validating {args.region}...")
    if args.region not in ["Altinova", "Heidel", "Valencia City", "Calpheon City"]:
        popjumppush = dict.fromkeys(
            [tuple(s.values())[0:3] for s in get_popjumppush_solutions(args.region)]
        )

        print("  CBC == popjumppush:", end=" ")
        if cbc == popjumppush:
            print("passed")
        else:
            cbc = set(cbc.keys())
            popjumppush = set(popjumppush.keys())
            diff1 = list(cbc - popjumppush)
            diff1 = sorted(diff1, key=lambda s: (s[0], s[1]))
            diff2 = list(popjumppush - cbc)
            diff2 = sorted(diff2, key=lambda s: (s[0], s[1]))
            print("failed")
            print(f"    popjumppush - CBC:\n  {list(popjumppush - cbc).sort()}")
            print(f"    CBC - popjumppush:\n  {list(cbc - popjumppush).sort()}")

        print("  HiGHS == popjumppush:", end=" ")
        if highs == popjumppush:
            print("passed")
        else:
            highs = set(highs.keys())
            popjumppush = set(popjumppush.keys())
            diff1 = list(highs - popjumppush)
            diff1 = sorted(diff1, key=lambda s: (s[0], s[1]))
            diff2 = list(popjumppush - highs)
            diff2 = sorted(diff2, key=lambda s: (s[0], s[1]))
            print("failed")
            print(f"    popjumppush - HiGHS:\n  {diff2}")
            print(f"    HiGHS - popjumppush:\n  {diff1}")
    else:
        print("  CBC == HiGHS:", end=" ")
        if cbc == highs:
            print("passed")
        else:
            cbc = set(cbc.keys())
            highs = set(highs.keys())
            diff1 = list(cbc - highs)
            diff1 = sorted(diff1, key=lambda s: (s[0], s[1]))
            diff2 = list(highs - cbc)
            diff2 = sorted(diff2, key=lambda s: (s[0], s[1]))
            print("failed")
            print(f"    CBC - HiGHS:\n  {cbc - highs}")
            print(f"    HiGHS - CBC:\n  {highs - cbc}")
    print()


def write_model(region: RegionInfo, _lodging, _storage):
    """writes out the model to an mps"""
    write_subset_selection_mps(
        region.items, region.item_reqs, region.weights, region.state_1_values, region.state_2_values
    )


def main(args):
    """Entry point"""

    if args.region == "ALL":
        with open("./data/houseinfo/" + "regioninfo.json", "r") as f:
            regions = json.load(f)

        for region in regions.values():
            args.region = region
            region_info = get_region_info(region)
            if len(region_info.buildings) == 0:
                print(f"No buildings for {region}...", region_info.buildings)
                continue
            if args.validate and not have_validation_files(region):
                print(f"Validation files for {region} must be generated/optimized.")
                continue
            if args.write:
                write_model(region_info, args.lodging, args.storage)
            elif args.all:
                optimize_all(args, region_info)
            else:
                solution = optimize(region_info, args.lodging, args.storage)
                print(
                    f"cost: {solution.cost}, lodging: {solution.lodging}, \
                          storage: {solution.storage}\n"
                    f"items: {solution.items}\n"
                    f"states: {solution.states}\n"
                )
    else:
        region_info = get_region_info(args.region)
        if args.write:
            write_model(region_info, args.lodging, args.storage)
        elif args.all:
            optimize_all(args, region_info)
        else:
            solution = optimize(region_info, args.lodging, args.storage)
            print(
                f"cost: {solution.cost}, lodging: {solution.lodging}, storage: {solution.storage}\n"
                f"items: {solution.items}\n"
                f"states: {solution.states}\n"
            )


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("-r", "--region", help="warehouse region")
    parser.add_argument("-l", "--lodging", help="desired lodging", type=int)
    parser.add_argument("-s", "--storage", help="desired storage", type=int)
    parser.add_argument(
        "-A",
        "--all",
        help="generate all dominant (ignores --lodging/--storage)",
        action=argparse.BooleanOptionalAction,
    )
    parser.add_argument(
        "-q", "--quiet", help="suppress listing", action=argparse.BooleanOptionalAction
    )
    parser.add_argument("-j", "--jobs", help="number of worker jobs to use", type=int, default=0)
    parser.add_argument(
        "-V",
        "--validate",
        help="validate against exact (only has meaning with --All)",
        action=argparse.BooleanOptionalAction,
    )
    parser.add_argument(
        "-W",
        "--write",
        help="write model to mps file (ignores all other arguments)",
        action=argparse.BooleanOptionalAction,
    )
    main(parser.parse_args())
