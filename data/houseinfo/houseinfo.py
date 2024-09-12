"""Module to read and parse a houseinfo.bss file to generate and write the houseinfo.json file."""

#  A good reference for this is `GameCommon/HouseInfoStaticStatus`.
#
#  To update the data use the bdo cli app from the project root:
#  ** Ensure that the dumpt CLIENT matches the latest version of the downloaded client!!!
#
#  $WS="$HOME\Workspaces"
#  $CLI = "$WS\rust\bdo\target\release\bdo.exe"
#  $BDO_ROOT = "C:\Program Files (x86)\BlackDesert"
#  $EXE = "$WS\bdo\client_dumps\BlackDesert64_DP_REBUILD_2023_03_08.exe"
#  $OUT = ".\data\houseinfo\"
#
#  . $CLI pad extract -f $BDO_ROOT\Paz --query-file houseinfo.bss --out-dir .
#  mv -force .\gamecommondata\binary\houseinfo.bss .\data\houseinfo\
#  rm .\gamecommondata\binary\
#  $DEST = $OUT + "Character.csv"
#  . $CLI client language-data -b $BDO_ROOT -f $EXE --group 6  --param3 0 -F param0,string -o $DEST
#  $DEST = $OUT + "HouseInfoReceipe.csv"
#  . $CLI client language-data -b $BDO_ROOT -f $EXE --group 16 --param3 0 -F param0,string -o $DEST
#  $DEST = $OUT + "RegionInfo.csv"
#  . $CLI client language-data -b $BDO_ROOT -f $EXE --group 17 --param3 0 -F param0,string -o $DEST
#  $DEST = $OUT + "Exploration.csv"
#  . $CLI client language-data -b $BDO_ROOT -f $EXE --group 29 --param3 0 -F param0,string -o $DEST
#  cd $OUT
#  python .\houseinfo.py -i houseinfo.bss -o houseinfo.json
#
#  Town counts:
#    Key     Town                     Ideals                      (w/warehouse & workers)
#    5       => "Velia"               => 186,624                  => 1,166,400
#    32      => "Heidel"              => 28,227,424,942,080       => 12,282,316,378,675,200
#    52      => "Glish"               => 14                       => 78
#    77      => "Calpheon City"       => (1.63e21 in python)      => (2.53e22 in python)
#    88      => "Olvia"               => 823,680                  => 40,365,000
#    107     => "Keplan"              => 8,064                    => 674,100
#    120     => "Port Epheria"        => 124,362                  => 40,777,724
#    126     => "Trent"               => 273                      => 16,100
#    182     => "Iliya Island"        => 240                      => 560
#    202     => "Altinova"            => 31,933,440,000           => 4,534,185,600,000
#    221     => "Tarif"               => 53,460                   => 396,900
#    229     => "Valencia City"       => (1.94e21 in python)      => (1.18e22 in python)
#    601     => "Shakatu"             => 36                       => 525
#    605     => "Sand Grain Bazaar"   => 144                      => 1,815
#    619     => "Ancado Inner Harbor" => 12                       => 33
#    693     => "Arehaza"             => 9                        => 32
#    694     => "Muiquun"             => 4                        => 4
#    706     => "Old Wisdom Tree"     => 4                        => 4
#    735     => "Grána"               => 524,288                  => 1,769,472
#    873     => "Duvencrune"          => 131,072                  => 663,552
#    955     => "O'draxxia"           => 6                        => 126
#    1124    => "Eilton"              => 24                       => 324

import argparse
import json
from kaitaistruct import KaitaiStream
from houseinfo_kaitaistruct import Houseinfo


def main(input_path, output_path):
    """Read and parse the houseinfo.bss file to generate and write the houseinfo.json file."""
    with open(input_path, "rb") as input_file:
        kstream = KaitaiStream(input_file)
        houseinfo = Houseinfo(kstream)

    house_info_list = []
    for house in houseinfo.house_info:
        # Skip the Manors.
        if house.character_key in [3814, 3815, 3841, 3842, 3847, 3848]:
            continue
        new_house = {}
        new_house["need_explore_point"] = house.need_explore_point
        new_house["affiliated_warehouse"] = house.affiliated_warehouse
        new_house["parent_node"] = house.parent_node
        new_house["character_key"] = house.character_key
        new_house["has_need_house_key"] = house.has_need_house_key
        need_house_key = house.need_house_key.need_house_key if house.has_need_house_key == 1 else 0
        new_house["need_house_key"] = need_house_key
        new_house["house_group"] = house.house_group
        new_house["house_floor"] = house.house_floor
        new_house["num_craft_list_items"] = house.num_craft_list_items
        new_craft_list = []
        for craft in house.craft_list:
            new_craft_list.append(
                {"house_level": craft.house_level, "item_craft_index": craft.item_craft_index}
            )
        new_house["craft_list"] = new_craft_list
        house_info_list.append(new_house)

    house_infos = {"house_info": house_info_list}
    with open(output_path, "w", encoding="UTF-8") as out_file:
        json.dump(house_infos, out_file, indent=4)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("-i", "--input", help="input path for bss file")
    parser.add_argument("-o", "--output", help="output path for json file")
    args = parser.parse_args()
    main(args.input, args.output)
