use super::houseinfo::*;
use crate::node_manipulation::*;
use anyhow::{Ok, Result};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub(crate) struct RegionNodes {
    pub region_name: String,
    #[allow(unused)]
    pub buildings: BuildingMap,
    pub num_nodes: usize,
    pub root: usize,
    pub parents: Vec<usize>,
    pub children: Vec<usize>,
    pub costs: Vec<usize>,
    pub states: Vec<usize>,
    pub state_values: HashMap<usize, Vec<usize>>,
    pub jump_indices: Vec<usize>,
    pub usage_counts: UsageCounters,
}

impl RegionNodes {
    pub fn new(buildings: &BuildingMap) -> Result<Self> {
        let region_name = buildings.first_key_value().unwrap().1.region_name.clone();
        let root = buildings.first_key_value().unwrap().1.region_key;
        let mut parents: Vec<usize> = Vec::with_capacity(buildings.len());
        let mut children: Vec<usize> = Vec::with_capacity(buildings.len());
        let mut costs: Vec<usize> = Vec::with_capacity(buildings.len());
        let mut states: Vec<usize> = Vec::with_capacity(buildings.len());
        let mut warehouse_counts = Vec::with_capacity(buildings.len());
        let mut worker_counts = Vec::with_capacity(buildings.len());

        parents.push(0);
        children.push(root);
        costs.push(0);
        states.push(0);
        warehouse_counts.push(0);
        worker_counts.push(0);

        for (key, building) in buildings.iter() {
            parents.push(building.need_key);
            children.push(*key);
        }
        let (parents, children) = arrange_largest_subtrees(root, &parents, &children, false);
        let (parents, children) = arrange_by_traversal_pre_order(root, &parents, &children);
        let jump_indices = generate_jump_indices(&parents, &children);

        let mut usage_counts = UsageCounters::new();
        for child in children.iter().skip(1) {
            let building = buildings.get(child).unwrap();
            costs.push(building.cost);
            warehouse_counts.push(building.warehouse_count);
            worker_counts.push(building.worker_count);
            if building.worker_count > 0 {
                states.push(2);
                usage_counts.worker_count += building.worker_count;
            } else {
                states.push(1);
                usage_counts.warehouse_count += building.warehouse_count;
            }
        }
        usage_counts.cost = costs.iter().sum();

        let mut state_values = HashMap::new();
        state_values.insert(1, warehouse_counts);
        state_values.insert(2, worker_counts);

        Ok(Self {
            region_name,
            buildings: buildings.clone(),
            num_nodes: children.len(),
            root,
            parents,
            children,
            costs,
            states,
            state_values,
            jump_indices,
            usage_counts,
        })
    }
}
