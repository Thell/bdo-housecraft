# Building Chains Calculator

- Exhaustively calculates exact node chain costs for warehouse and workers.
- Data is written to a file `/data/housecraft/region_name.csv` containing
    warehouse_count, worker_count, building_usage, warehouse_usage, worker_usage
  sorted by warehouse and worker count in ascending order.

The `_usage` values are the combinadics rank of the buildings, warehouse and worker usages when the region's building nodes are preorder and sorted with the largest subtrees to the right with binary indexing from left to right.

> **Note**: Calpheon City and Valencia City are **not** included in the exact results.

## Retrieval Usage
To retrieve a particular housecraft usage for a given city, warehouse count and worker count.

```shell
housecraft -R "Velia" -S 12 -L 3
```

If the given combination of warehouse and worker is not possible the next highest cost entry will be used that has at least that many warehouse and worker assignments.

## Generation Usage
To generate all of the exact housecraft usages for a given city.

```shell
housecraft --region Velia --generate --progress
```
