import argparse
from collections import namedtuple
from optimize_key_selection import optimize_subset_selection as optimize

items = [
    202, 2895, 2896, 2897, 2900, 2901, 2902, 2898, 2870, 2873, 2874, 2871, 2875, 2876, 2877, 2878,
    2879, 2881, 2882, 2883, 2884, 2885, 2889, 2891, 2892, 2893, 2894, 2866, 2867, 2869, 2906, 2907,
    2908, 2909, 2911, 2912, 2913, 2914, 2915, 2916, 2904, 2903, 2831, 2832, 2833, 2842, 2843, 2844,
    2846, 2847, 2848, 2886, 2887, 2888, 2834, 2835, 2845, 2910
]

item_reqs = [
    0, 202, 2895, 2896, 2897, 2900, 2901, 2897, 202, 2870, 2873, 2870, 202, 2875, 2876, 2877, 2878,
    202, 2881, 2882, 2883, 2884, 202, 2889, 2891, 2892, 2893, 202, 2866, 2866, 202, 2906, 2907,
    2908, 202, 2911, 2912, 2913, 202, 2915, 2916, 2904, 202, 2831, 2832, 202, 2842, 2843, 202, 2846,
    2847, 202, 2886, 2887, 202, 2834, 202, 202
]

weights = [
    0, 1, 2, 1, 3, 2, 4, 4, 1, 3, 5, 4, 2, 2, 3, 3, 4, 1, 3, 2, 2, 3, 1, 3, 2, 2, 3, 2, 2, 4, 2, 2,
    4, 5, 2, 3, 2, 5, 2, 3, 3, 5, 1, 1, 2, 1, 1, 2, 1, 1, 2, 2, 3, 5, 1, 1, 2, 5
]

state_1_values = [
    0, 3, 5, 3, 8, 8, 16, 12, 3, 8, 12, 8, 3, 5, 8, 8, 16, 3, 5, 5, 8, 16, 3, 5, 5, 8, 16, 3, 5, 8,
    3, 5, 8, 16, 3, 5, 8, 12, 3, 5, 8, 16, 3, 3, 5, 3, 3, 5, 3, 3, 5, 5, 5, 12, 3, 3, 5, 12
]

state_2_values = [
    0, 1, 2, 2, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 1, 2, 0, 0, 0, 0, 0, 0, 0, 1, 2, 0, 0, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0
]


def main(_region, storage, lodging):
    Solution = namedtuple("Solution", 'cost storage lodging items states')
    solution = Solution(
        *optimize(items, item_reqs, weights, state_1_values, state_2_values, storage, lodging))
    print(solution)


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument("-r", "--region", help="warehouse region")
    parser.add_argument("-l", "--lodging", help="desired lodging", type=int)
    parser.add_argument("-s", "--storage", help="desired storage", type=int)
    args = parser.parse_args()
    main(args.region, args.storage, args.lodging)
