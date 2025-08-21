# About

This project is an implementation of the pop_jump_push algorithm. It uses graph
data from the MMORPG Black Desert Online's town building networks, which provide
varying graph sizes and multistate nodes, to generate and score exact solutions
to the problem of minimizing cost while maximizing utilization given minimum
usage levels.

In Black Desert Online the map is split into regions, each region consists of
towns and the towns have buildings in chains where a building can be required to
be rented prior to another building in the chain (an arborescence). The
buildings can be used for many purposes and this project only concerns itself
with worker lodging and warehouse storage.

# Required input files

The following files are required in `\data\houseinfo\`:
- `houseinfo.json` describes the properties of houses. is a dictionary but the keys are not used. each value describes a house and must contain all of the following fields:
  - `need_explore_point: <integer>` the CP cost of the house
  - `affiliated_town: <townkey>` which region the house belongs to
  - `parent_node: <nodekey>` required but unused
  - `character_key: <charkey>` is the actual key to perform linking
  - `len_need_house_key` either 1 if the house has a requirement, or 0 if not
  - `need_house_key` either a `charkey` of a required house, or 0
  - `house_group` required but unused
  - `house_floor` required but unused
  - `craft_list` a list of possible uses. each use must have fields:
	- `item_craft_index: <houseusekey>` the intended use, where 1 is lodging and 2 is storage
    - `house_level: <integer>` the "level" of usage. the level defines the amount of lodging (level 1: 1 bed, 2: 2, 3: 4, 4: 6, 5: 8) or storage (level 1: 3 storage cells, 2: 5, 3: 8, 4: 12, 5: 16) the house can give. this mapping is not configurable.
  - `num_craft_list_items` the length of `craft_list`
- Localization strings (use UTF8 for entries like Grána). First line must contain `Param0,String` header, following lines are just `key,value`:
  - `Region.csv` - names of regions (`townkey`s)
  - `Exploration.csv` - names of nodes (`nodekey`s)
  - `Character.csv` - names of houses (`charkey`s)
  - `HouseInfoReceipe.csv` - human readable decriptions of "house use" integers (`houseusekey`s)
- `regioninfo.json` must contain a dictionary of regions to work on (`{<townkey>: name, ...}`, names are not used)

Use `/python/regenerate_houseinfo_data.py` as guidance for preparing the input files yourself.
	

# Building Information

## List building chains.

When one chain can provide the same or more lodging/storage for less or the same cost it is said to 'dominate' another chain. Multiple chains can be dominant for a cost such that one may have more or less lodging/storage than the other but both have the same or more than the requested counts.

```md
> housecraft -R "Velia" -S 12 -L 3

  Building                    🪙   📦   👷
────────────────────────────────────────────
  Velia 1, Rm. 1              1         1
  Velia 1, Rm. 2              1         2
  Velia 1, Rm. 3              1    5
  Balenos 1-1, Bartali Farm   2    5
  Balenos 1-2, Bartali Farm   1    5
  Totals                      6    15   3  
════════════════════════════════════════════

  Building                    🪙   📦   👷
────────────────────────────────────────────
  Velia 1, Rm. 1              1    3
  Velia 1, Rm. 2              1         2
  Velia 1, Rm. 3              1         2
  Balenos 1-1, Bartali Farm   2    5
  Balenos 1-2, Bartali Farm   1    5
  Totals                      6    13   4  
════════════════════════════════════════════
```

## List building usages and counts.

```md
> housecraft --list-crafts -R Shakatu

Shakatu
  Crafting Usage        Count 
───────────────────────────────
  Crop Factory 1        1
  Fish Factory 1        1
  Lodging 1             3
  Lodging 2             2
  Lodging 3             1
  Mineral Workshop  1   1
  Mushroom Factory 1    2
  Refinery 1            1
  Refinery 2            2
  Residence 1           7
  Storage 1             2
  Storage 2             2
  Storage 3             2
  Storage 4             1
  Tool Workshop 1       1
  Tool Workshop 2       1
  Wood Workshop  1      1
═══════════════════════════════
```

## Find buildings for a usage.

```md
> housecraft --find-craft "Wood Workshop 2"


Duvencrune
  Crafting Usage     Key    Building                 Cost 
───────────────────────────────────────────────────────────
  Wood Workshop  2   3615   Dormann Lumber Camp 2    7
  Wood Workshop  2   3616   Khimut Lumber Camp 1-1   5
═══════════════════════════════════════════════════════════

Eilton
  Crafting Usage     Key    Building        Cost 
──────────────────────────────────────────────────
  Wood Workshop  2   3821   Camp Balacs 1   3
══════════════════════════════════════════════════

Grána
  Crafting Usage     Key    Building   Cost 
─────────────────────────────────────────────
  Wood Workshop  2   3510   Grána 9    6
═════════════════════════════════════════════

O'draxxia
  Crafting Usage     Key    Building      Cost 
────────────────────────────────────────────────
  Wood Workshop  2   3803   O'draxxia 3   3
════════════════════════════════════════════════
```

## List region information.

```md
> housecraft --list-regions

  Region                CP     Storage   Lodging 
──────────────────────────────────────────────────
  Altinova              143    386       18
  Ancado Inner Harbor   13     36        3
  Arehaza               8      26        3
  Calpheon City         585    1703      94
  Duvencrune            96     272       7
  Eilton                22     112       11
  Glish                 16     57        4
  Grána                 132    288       8
  Heidel                156    416       27
  Iliya Island          19     59        2
  Keplan                60     163       10
  Muiquun               8      32        0
  O'draxxia             16     80        13
  Old Wisdom Tree       5      32        0
  Olvia                 50     151       15
  Port Epheria          60     188       28
  Sand Grain Bazaar     24     67        7
  Shakatu               18     44        11
  Tarif                 31     86        7
  Trent                 38     109       9
  Valencia City         340    1116      46
  Velia                 45     117       7
  Totals                1885   5540      330     
══════════════════════════════════════════════════
```

# Generate building chains.

- Exhaustively calculates exact node chain costs for warehouse and workers.
- Data is written to a file `/data/housecraft/{region_name}.json` containing storage, lodging, building and usage states sorted by warehouse and worker count in ascending order.
> **Note: Calpheon City, Valencia City and Heidel are _not_ included in the exact results.**

```md
> housecraft.exe --generate -R Velia
Generating values for Velia consisting of 23 buildings in 186624 chains with 1166400 storage/lodging combinations
With a maximum cost of 45 with 7 lodging and 98 storage (out of 117 possible).
[2023-04-02T08:56:59.946816800Z] generating...
  [2023-04-02T08:56:59.954274Z] Visited 1166400 combinations.
[2023-04-02T08:56:59.954342300Z] retaining...
Captured chain count: 795
[2023-04-02T08:56:59.954715Z] writing...
Result: 252 'best of best' scored storage/lodging chains written to ./data/housecraft/Velia.json
```

# Optimize building chains.

- Calculates node chain costs for warehouse and workers using HiGHS.
- Data is written to a file `/data/housecraft/{region_name}.json` containing storage, lodging, building and usage states sorted by warehouse and worker count in ascending order.

```md
> housecraft.exe --optimize -R Velia
[2023-04-25T22:56:47.468527300Z INFO  housecraft] Start up
[2023-04-25T22:56:47.468602200Z INFO  housecraft::optimize] preparing...
Optimizing values for Velia consisting of 23 buildings in 186624 chains with 1166400 storage/lodging combinations
With a maximum cost of 45 with 7 lodging and 98 storage (out of 117 possible).
[2023-04-25T22:56:47.473279400Z INFO  housecraft::optimize] optimizing...
[2023-04-25T22:56:47.474107800Z INFO  housecraft::optimize] START: Job 0 with 944 combinations on 24 nodes using 72 cols and 37 rows.
[2023-04-25T22:56:48.237550900Z INFO  housecraft::optimize] COMPLETE: Job 0 with 944 combinations with 878 feasible yielding 878 chains.
[2023-04-25T22:56:48.237850100Z INFO  housecraft::optimize] merging...
[2023-04-25T22:56:48.238293500Z INFO  housecraft::optimize] Captured chain count: 878
[2023-04-25T22:56:48.238523600Z INFO  housecraft::optimize] retaining...
[2023-04-25T22:56:48.239145700Z INFO  housecraft::optimize] Retained chain count: 252
[2023-04-25T22:56:48.239284200Z INFO  housecraft::optimize] writing...
Result: 252 'best of best' scored storage/lodging chains written to ./data/housecraft/Velia.json.
[2023-04-25T22:56:48.240876000Z INFO  housecraft] Complete
```

## Generation/Optimization Notes

Even though the generator visits _all_ combinations of all buildings in all
states and the optimizer visits all combinations of storage and lodging they
both yield only the dominating chains.

> A dominant chain strictly dominates a different chain when it provides the
same or more workers and or warehouse counts for less or the same cost.

The resulting chains are the best-of-the-best chains for any combination of
cost, worker and warehouse counts.

All regions except Heidel, Valencia City and Calpheon City have had exact
results generated.

The exact results validated the results of the both the Python implmentated
optimizer using CBC and the Rust implemented optimizer using HiGHS with 100%
matches. Then both CBC and HiGHS optimized the regions of Heidel, Valencia City
and Calpheon City with only a single resulting difference.
That difference was on an instance that would never happen in reality; it
utilized 6 lodging, and either 399 or 400 storage slots both with a cost of 152
when the maximum usable lodging in-game is 192 per region.


# Building

External requirements for building on Windows:

- cmake
- clang
- libz
- ninja
