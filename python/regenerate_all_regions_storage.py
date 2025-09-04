"""
This file should be called __after__ `regenerate_houseinfo_data.py` since it reads '`all_lodging_storage.json`'.
"""

import json
from pathlib import Path


def package_path() -> Path:
    import importlib.resources

    with importlib.resources.as_file(importlib.resources.files("houseinfo")) as path:
        return path


def data_path() -> Path:
    return package_path().parent / "data" / "houseinfo"


def format_all_regions_storage(data):
    from compact_json import Formatter
    import re

    formatter = Formatter(
        ensure_ascii=False,
        omit_trailing_whitespace=True,
        always_expand_depth=0,
        max_inline_length=99999,
    )
    json_data = formatter.serialize(data)
    json_data = re.sub(r"\s+,", ",", json_data)
    json_data = re.sub(r"\s+}", "}", json_data)
    json_data = re.sub(r"\s+\]", "]", json_data)

    path = data_path().parent / "housecraft" / "all_regions_storage.json"
    with open(path, "w", encoding="utf-8") as f:
        f.write(json_data)


def extract_storage_only_chains(data):
    """Extract all region '0' lodging lists as storage-only chains."""
    storage_only = []
    for _region, lodgings in data.items():
        if "0" in lodgings:
            for c in lodgings["0"]:
                if c["storage"] == 0:
                    continue
                chain = {"totalStorage": 0, "totalCost": 0}
                chain.update(dict(c))
                storage_only.append(chain)
    return storage_only


def vested_dominance_merge(chains):
    """
    Merge chains from all regions into a single dominance-sorted list based on vested contribution.
    """
    region_storage = {}
    region_cost = {}
    remaining_chains = chains.copy()
    merged = []

    while remaining_chains:
        best_index = None
        best_ratio = -1

        for i, chain in enumerate(remaining_chains):
            region = chain["indices"][0]

            incr_storage = chain["storage"] - region_storage.get(region, 0)
            incr_cost = chain["cost"] - region_cost.get(region, 0)
            if incr_storage <= 0 or incr_cost < 0:
                # Already fully vested - simply skip rather than remove
                continue

            ratio = incr_storage / incr_cost if incr_cost > 0 else float("inf")
            if ratio > best_ratio:
                best_ratio = ratio
                best_index = i

            # Tie-breaker: if ratios equal, prefer lower incremental cost
            elif ratio == best_ratio:
                if incr_cost < (
                    remaining_chains[best_index]["cost"]
                    - region_cost.get(remaining_chains[best_index]["indices"][0], 0)
                ):
                    best_index = i
                elif chain["indices"][0] < remaining_chains[best_index]["indices"][0]:
                    best_index = i

        if best_index is None:
            break

        # Move and update the best chain
        chain = remaining_chains.pop(best_index)
        region = chain["indices"][0]
        incr_storage = chain["storage"] - region_storage.get(region, 0)
        incr_cost = chain["cost"] - region_cost.get(region, 0)

        region_storage[region] = chain["storage"]
        region_cost[region] = chain["cost"]

        chain["total_storage"] = sum(region_storage.values())
        chain["total_cost"] = sum(region_cost.values())

        merged.append(chain)

    return merged


def generate_all_regions_storage_json():
    path = data_path().parent / "housecraft" / "all_lodging_storage.json"
    with open(path, "r") as f:
        json_data = json.load(f)

    storage_only = extract_storage_only_chains(json_data)
    merged_chain = vested_dominance_merge(storage_only)
    format_all_regions_storage(merged_chain)


if __name__ == "__main__":
    generate_all_regions_storage_json()
