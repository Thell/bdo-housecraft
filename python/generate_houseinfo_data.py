"""Module to extract and prepare data."""
# End users should not need to use this file.

# bdo-cli is not publicly distributed but you can use tools such as BDOcrypt to extract
# language data and PAZ-unpacker to extract binary files.


# NOTE:
# Execute this module to update data after new regions with updated houseinfo are added.
# This module requires:
#   bdo-cli: which provides pad and language data extraction requiring SCY rebuild.
#   kaitai-compiler: to compile binary structure formats when they have changed. (use flag --update)

# Will use

import atexit
from functools import lru_cache
from pathlib import Path

# Global cache of offsets for all binary files with an associated offsets file.
OFFSETS_CACHE = {}

# Global cache of records for some binary files without an associated offsets file.
RECORDS_CACHE = {}

# Global cache of file handles for non-mmap binary files
FILE_HANDLES = {}


@lru_cache(maxsize=None)
def package_path() -> Path:
    import importlib.resources

    with importlib.resources.as_file(importlib.resources.files("houseinfo")) as path:
        return path


@lru_cache(maxsize=None)
def data_path() -> Path:
    return package_path().parent / "data" / "houseinfo"


@lru_cache(maxsize=None)
def binary_path() -> Path:
    return data_path() / "gamecommondata" / "binary"


@lru_cache(maxsize=None)
def format_path() -> Path:
    return data_path() / "formats"


@lru_cache(maxsize=None)
def parser_path() -> Path:
    return package_path() / "parsers"


@lru_cache(maxsize=None)
def strings_path() -> Path:
    return data_path()


def scy_rebuild_path():
    import os
    from tkinter import filedialog

    scy_rebuild = filedialog.askopenfilename(
        title="SCY_REBUILD File Selector", filetypes=[("Executable Files", "*.exe")]
    )
    os.environ["SCY_REBUILD"] = scy_rebuild
    return scy_rebuild


def get_binary_file_handle(file_path):
    global FILE_HANDLES

    file_key = str(file_path)

    if file_key not in FILE_HANDLES:
        FILE_HANDLES[file_key] = open(binary_path() / file_path, "rb")

    return FILE_HANDLES[file_key]


def close_all_file_handles():
    global FILE_HANDLES
    for handle in FILE_HANDLES.values():
        handle.close()
    FILE_HANDLES.clear()


def bdo_cli_path() -> Path:
    from os import environ

    return Path(environ.get("USERPROFILE", "")) / ".cargo" / "bin" / "bdo.exe"


def kaitai_struct_compile():
    from subprocess import run

    print("Compiling kaitai format files into parsers...")
    try:
        args = [
            "kaitai-struct-compiler",
            "-t",
            "python",
            "--outdir",
            f"{parser_path().as_posix()}",
            f"{format_path().joinpath('*.ksy')}",
        ]
        run(args=args, shell=True, capture_output=True)
    except Exception as e:
        print("failed", e)


def extract_binary(filename):
    from os import environ
    from subprocess import run

    bdo_cli = bdo_cli_path()
    bdo_root = Path(environ.get("BDO_ROOT", r"C:\Program Files (x86)\BlackDesert"))
    paz_root = bdo_root / "Paz"
    print(f"Extracting {filename}")
    try:
        run(
            args=[
                f"{bdo_cli.as_posix()}",
                "pad",
                "extract",
                "--from-dir",
                f"{paz_root.as_posix()}",
                "--query-file",
                f"^{filename}$",
                "--out-dir",
                f"{data_path().as_posix()}",
            ]
        )
    except Exception as e:
        print("failed", e)


def ensure_binary(path):
    if not path.is_file():
        extract_binary(path.name)


def extract_meta_info(format_file):
    file_id = None
    file_ext = None

    with open(format_file, "r") as file:
        for line in file:
            if "id:" in line and file_id is None:
                file_id = line.split(":", 1)[1].strip()
            if "file-extension:" in line and file_ext is None:
                file_ext = line.split(":", 1)[1].strip()
            if file_id and file_ext:
                break
    return binary_path() / f"{file_id}.{file_ext}"


def extract_strings(language_data_group, param0=any, param1=0, param2=0, param3=0):
    """
    language_data_group is case sensitive and is a regex string.
    Prefix with (?i) to be case insentitive.
    """

    from os import environ
    from subprocess import run

    bdo_cli = bdo_cli_path()
    bdo_root = Path(environ.get("BDO_ROOT", r"C:\Program Files (x86)\BlackDesert"))
    scy_rebuild = environ.get("SCY_REBUILD", None)
    if scy_rebuild is None:
        scy_rebuild = Path(scy_rebuild_path()).as_posix()
    else:
        scy_rebuild = Path(scy_rebuild).as_posix()

    args = [
        f"{bdo_cli.as_posix()}",
        "client",
        "language-data",
        "--bdo-root",
        f"{bdo_root}",
        "--from-file",
        f"{scy_rebuild}",
    ]

    if isinstance(language_data_group, int):
        args.append("--group")
        args.append(str(language_data_group))
    if isinstance(language_data_group, str):
        args.append("--query-group")
        args.append(language_data_group)

    if param0 != any:
        args.append("--param0")
        args.append(f"{param0}")

    args.extend(
        [
            "--param1",
            f"{param1}",
            "--param2",
            f"{param2}",
            "--param3",
            f"{param3}",
        ]
    )

    args.extend(
        [
            "--out-file",
            f"{(strings_path() / f"{language_data_group}.csv").as_posix()}",
            "-F",
            "param0,string",
        ]
    )
    print(f"Extracting {language_data_group} strings...")
    try:
        run(args)
    except Exception as e:
        print("failed", e)


def remove_files(path):
    for file in path().iterdir():
        if file.is_file():
            file.unlink()


def ensure_data_availability():
    def get_stems(path):
        return [
            p.stem
            for p in path.iterdir()
            if not p.stem.startswith(".") and not p.stem.startswith("_")
        ]

    formats = sorted(get_stems(format_path()))
    parsers = sorted(get_stems(parser_path()))
    if formats != parsers:
        print("formats != parsers\n", formats, "\n", parsers)
        kaitai_struct_compile()

    binaries = get_stems(binary_path())
    binaries = sorted(binaries)
    if formats != binaries:
        print("formats != binaries\n", formats, "\n", binaries)
        for path in format_path().iterdir():
            if path.stem.startswith("."):
                continue
            file = extract_meta_info(path)
            ensure_binary(file)


def dict_to_camel_case(data):
    import inflection

    if isinstance(data, dict):
        new_dict = {}
        for key, value in data.items():
            new_key = inflection.camelize(key, False)
            new_dict[new_key] = dict_to_camel_case(value)
        return new_dict
    elif isinstance(data, list):
        return [dict_to_camel_case(item) for item in data]
    else:
        return data


def write_json(json_data, file_path, format, camelize_keys=False):
    import json

    if camelize_keys:
        json_data = dict_to_camel_case(json_data)

    if format:
        from compact_json import Formatter

        formatter = Formatter(ensure_ascii=False, omit_trailing_whitespace=True)
        formatter.dump(json_data, output_file=file_path, newline_at_eof=True)
    else:
        with open(file_path, "w") as out_file:
            json.dump(json_data, out_file, indent=4)


def houseinfo_table_reader():
    from parsers.houseinfo import Houseinfo

    return Houseinfo.from_file(binary_path() / "houseinfo.bss").house_info


def mansionlandinfo_table_reader():
    from parsers.mansionlandinfo import Mansionlandinfo

    return Mansionlandinfo.from_file(binary_path() / "mansionlandinfo.bss").mansionlandinfo_table


def regions_table_reader():
    from parsers.regioninfo import Regioninfo

    return Regioninfo.from_file(binary_path() / "regioninfo.bss").regioninfo_table


def print_short_output(houseinfo):
    print("\nProcessing houseinfo data resulted in:")
    print("houses:", len(houseinfo))


def extract_mansion_character_keys():
    mansionlandinfo = [r.land_character_key for r in mansionlandinfo_table_reader()]
    mansionlandinfo.extend([r.building_character_key for r in mansionlandinfo_table_reader()])
    return sorted(mansionlandinfo)


def extract_en_strings():
    for group_name in ["Character", "Exploration", "HouseInfoReceipe", "Region"]:
        extract_strings(group_name)


def generate_houseinfo_data():
    from natsort import natsorted

    mansion_character_keys = extract_mansion_character_keys()

    houseinfo = {}
    for house in houseinfo_table_reader():
        if house.character_key in mansion_character_keys:
            continue

        craft_list = [
            {"house_level": craft.house_level, "item_craft_index": craft.item_craft_index}
            for craft in house.craft_list
        ]

        houseinfo[str(house.character_key)] = {
            "need_explore_point": house.need_explore_point,
            "affiliated_town": house.affiliated_town,
            "parent_node": house.parent_node,
            "character_key": house.character_key,
            "len_need_house_key": house.len_need_house_key,
            "need_house_key": house.need_house_key[0] if house.len_need_house_key else 0,
            "house_group": house.house_group,
            "house_floor": house.house_floor,
            "num_craft_list_items": house.num_craft_list_items,
            "craft_list": craft_list,
        }

    return dict(natsorted(houseinfo.items()))


def generate_regioninfo_data(strings_filestem):
    import csv
    from natsort import natsorted

    with open(strings_path() / f"{strings_filestem}.csv", "r", encoding="UTF-8") as f:
        strings = {int(entry["Param0"]): entry["String"] for entry in csv.DictReader(f)}

    regioninfo = {}
    for region in regions_table_reader():
        if region.npc_worker_character_key > 0 or region.warehouse_character_key > 0:
            regioninfo[str(region.id)] = strings[region.id]

    return dict(natsorted(regioninfo.items()))


def do_housecraft_optimize():
    """cargo build and then optimize"""
    from subprocess import run

    args = ["cargo", "build", "--release"]
    try:
        run(args)
    except Exception as e:
        print("failed cargo build", e)

    args = ["target/debug/housecraft", "--optimize", "-R", "ALL", "--limit-warehouse", "184"]
    try:
        run(args)
    except Exception as e:
        print("failed optimization", e)


def format_all_lodging_storage():
    from compact_json import Formatter
    import json
    import re

    path = data_path().parent / "housecraft" / "all_lodging_storage.json"
    with open(path, "r") as f:
        json_data = json.load(f)

    formatter = Formatter(
        ensure_ascii=False,
        omit_trailing_whitespace=True,
        always_expand_depth=1,
        max_inline_length=99999,
    )
    json_data = formatter.serialize(json_data)
    json_data = re.sub(r"\s+,", ",", json_data)
    json_data = re.sub(r"\s+}", "}", json_data)
    json_data = re.sub(r"\s+\]", "]", json_data)
    with open(path, "w") as f:
        f.write(json_data)


def main(args):
    if args.update:
        remove_files(parser_path())
        remove_files(binary_path())
    ensure_data_availability()

    extract_en_strings()
    houseinfo = generate_houseinfo_data()
    regioninfo = generate_regioninfo_data("Region")

    print_short_output(houseinfo)
    write_json(regioninfo, data_path() / "regioninfo.json", format=True, camelize_keys=False)
    write_json(houseinfo, data_path() / "houseinfo.json", format=True, camelize_keys=False)

    do_housecraft_optimize()
    format_all_lodging_storage()


atexit.register(close_all_file_handles)

if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser()
    parser.add_argument("-U", "--update", help="Update parsers from formats.")
    args = parser.parse_args()
    main(args)
