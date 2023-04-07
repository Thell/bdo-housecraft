""" houseinfo.json extraction/parsing functions.
"""

from csv import DictReader
import json


# pypy can't use match statements yet.
def get_affiliated_town_key(name):
    """ Return town id number given town name.
    """
    if name == "Velia":
        return 5
    if name == "Heidel":
        return 32
    if name == "Glish":
        return 52
    if name == "Calpheon City":
        return 77
    if name == "Olvia":
        return 88
    if name == "Keplan":
        return 107
    if name == "Port Epheria":
        return 120
    if name == "Trent":
        return 126
    if name == "Iliya Island":
        return 182
    if name == "Altinova":
        return 202
    if name == "Tarif":
        return 221
    if name == "Valencia City":
        return 229
    if name == "Shakatu":
        return 601
    if name == "Sand Grain Bazaar":
        return 605
    if name == "Ancado Inner Harbor":
        return 619
    if name == "Arehaza":
        return 693
    if name == "Muiquun":
        return 694
    if name == "Old Wisdom Tree":
        return 706
    if name == "Grána":
        return 735
    if name == "Duvencrune":
        return 873
    if name == "O'draxxia":
        return 955
    if name == "Eilton":
        return 1124
    return 0


def get_worker_count(craft_list):
    """ Return worker count from house level given crafting list.
    """
    level = 0
    for craft in craft_list:
        if craft.get("item_craft_index") == 1:
            level = craft.get("house_level")

    if level == 1:
        return 1
    if level == 2:
        return 2
    if level == 3:
        return 4
    if level == 4:
        return 6
    if level == 5:
        return 8
    return 0


def get_extend_warehouse_count(craft_list):
    """ Return warehouse count from house level given crafting list.
    """
    level = 0
    for craft in craft_list:
        if craft.get("item_craft_index") == 2:
            level = craft.get("house_level")

    if level == 1:
        return 3
    if level == 2:
        return 5
    if level == 3:
        return 8
    if level == 4:
        return 12
    if level == 5:
        return 16
    return 0


def get_extend_stable_count(craft_list):
    """ Return stable count from house level given crafting list.
    """
    level = 0
    for craft in craft_list:
        if craft.get("item_craft_index") == 3:
            level = craft.get("house_level")

    if level == 1:
        return 1
    if level == 2:
        return 2
    if level == 3:
        return 3
    if level == 4:
        return 4
    if level == 5:
        return 5
    return 0


def read_csv_data(filename):
    """ Read csv data file from the ./data directory.
    """
    path = "./data/houseinfo/" + filename
    records = {}
    with open(path, encoding="UTF-8") as file:
        for record in DictReader(file):
            records[int(record["Param0"])] = record["String"]
    return records


def get_town_buildings_by_key(town_key):
    """ Return all building for a town given the town id key.
    """
    character = read_csv_data("Character.csv")
    exploration = read_csv_data("Exploration.csv")
    _usage = read_csv_data("HouseInfoReceipe.csv")
    region = read_csv_data("RegionInfo.csv")

    with open("./data/houseinfo/houseinfo.json", encoding="UTF-8") as file:
        house_infos = json.load(file)

    buildings = {}
    for house_info in house_infos["house_info"]:
        if house_info.get("affiliated_warehouse") == town_key:
            key = house_info.get("character_key")
            need_key = house_info.get("need_house_key") if house_info.get(
                "has_need_house_key") == 1 else house_info.get("affiliated_warehouse")
            node_key = house_info.get("parent_node")
            region_key = house_info.get("affiliated_warehouse")

            building_name = character.get(key)
            node_name = exploration.get(node_key)
            region_name = region.get(region_key)

            cost = house_info.get("need_explore_point")
            worker_count = get_worker_count(house_info.get("craft_list"))
            warehouse_count = get_extend_warehouse_count(house_info.get("craft_list"))
            stable_count = get_extend_stable_count(house_info.get("craft_list"))

            building = {
                "key": key,
                "need_key": need_key,
                "node_key": node_key,
                "region_key": region_key,
                "building_name": building_name,
                "node_name": node_name,
                "region_name": region_name,
                "cost": cost,
                "worker_count": worker_count,
                "stable_count": stable_count,
                "warehouse_count": warehouse_count,
            }
            buildings[key] = building
    return buildings


def get_town_buildings_by_name(town_name):
    """ Return all buildings for a town given the town name.
    """
    town_key = get_affiliated_town_key(town_name)
    return get_town_buildings_by_key(town_key)


def get_region_buildings(region_name):
    """ Prepare town buildings as a town hierarchy with parents, children, states and values
    for both warehouse and worker slots.
    """
    town_key = get_affiliated_town_key(region_name)
    if town_key == 0:
        raise SystemExit(f"{region_name} was not found.")
    buildings = get_town_buildings_by_name(region_name)
    keys = [town_key]
    needed_keys = [0]
    costs = [0]
    state_1_values = [0]
    state_2_values = [0]
    for (key, value) in buildings.items():
        keys.append(key)
        needed_keys.append(value.get("need_key"))
        costs.append(value.get("cost"))
        state_1_values.append(value.get("warehouse_count"))
        state_2_values.append(value.get("worker_count"))
    return buildings, keys, needed_keys, costs, state_1_values, state_2_values
