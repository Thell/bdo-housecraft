# ** Work in progress **

# About

This project is an implementation of the pop_jump_push algorithm. It uses graph data from the MMORPG Black Desert Online's town building networks, which provide
varying graph sizes and multistate nodes, to generate and score exact solutions to the problem of minimizing cost while maximizing utilization given minimum usage levels.

In Black Desert Online the map is split into regions, each region consists of towns and the towns have buildings in chains where a building can be required to be rented prior to another building in the chain (an arborescence). The buildings can be used for many purposes and this project only concerns itself with worker lodging and warehouse storage.

# Building Information

> **Note: Calpheon City, Valencia City and Heidel are _not_ included in the exact results.**

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
  Mineral Workbench 1   1
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
  Wood Workbench 1      1
═══════════════════════════════
```

## Find buildings for a usage.

```md
> housecraft --find-craft "Wood Workbench 2"


Duvencrune
  Crafting Usage     Key    Building                 Cost 
───────────────────────────────────────────────────────────
  Wood Workbench 2   3615   Dormann Lumber Camp 2    7
  Wood Workbench 2   3616   Khimut Lumber Camp 1-1   5
═══════════════════════════════════════════════════════════

Eilton
  Crafting Usage     Key    Building        Cost 
──────────────────────────────────────────────────
  Wood Workbench 2   3821   Camp Balacs 1   3
══════════════════════════════════════════════════

Grána
  Crafting Usage     Key    Building   Cost 
─────────────────────────────────────────────
  Wood Workbench 2   3510   Grána 9    6
═════════════════════════════════════════════

O'draxxia
  Crafting Usage     Key    Building      Cost 
────────────────────────────────────────────────
  Wood Workbench 2   3803   O'draxxia 3   3
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