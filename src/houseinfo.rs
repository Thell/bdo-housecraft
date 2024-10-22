use anyhow::{bail, Context, Ok, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::path::Path;

pub(crate) type IndexedStringMap = HashMap<usize, String>;
pub(crate) type BuildingMap = BTreeMap<usize, Building>;
pub(crate) type RegionBuildingMap = BTreeMap<String, BuildingMap>;
pub(crate) type CraftBuildingMap = BTreeMap<String, (usize, Vec<Building>)>;

// pub const HOUSECRAFT_TABLE_STYLE: &str = "0123456789abcdefghi";
pub(crate) const HOUSECRAFT_TABLE_STYLE: &str = "   ═────      ═  ══";

#[derive(Clone, Debug)]
pub(crate) struct UsageCounters {
    pub cost: usize,
    pub warehouse_count: usize,
    pub worker_count: usize,
}

impl UsageCounters {
    pub fn new() -> Self {
        Self {
            cost: 0,
            warehouse_count: 0,
            worker_count: 0,
        }
    }
}

#[allow(unused)]
#[derive(Clone, Debug)]
pub(crate) struct Building {
    pub key: usize,
    pub need_key: usize,
    pub node_key: usize,
    pub region_key: usize,
    pub building_name: String,
    pub node_name: String,
    pub region_name: String,
    pub cost: usize,
    pub worker_count: usize,
    pub stable_count: usize,
    pub warehouse_count: usize,
    pub craft_list: Vec<CraftList>,
}

impl Building {
    pub fn new(
        house_info: &HouseInfo,
        character: &IndexedStringMap,
        exploration: &IndexedStringMap,
        region: &IndexedStringMap,
    ) -> Self {
        let key = house_info.character_key;
        let node_key = house_info.parent_node;
        let region_key = house_info.affiliated_town;

        Self {
            key,
            need_key: if house_info.len_need_house_key == 1 {
                house_info.need_house_key
            } else {
                house_info.affiliated_town
            },
            node_key,
            region_key,

            building_name: character.get(&key).unwrap().to_string(),
            node_name: exploration.get(&node_key).unwrap().to_string(),
            region_name: region.get(&region_key).unwrap().to_string(),

            cost: house_info.need_explore_point as usize,
            worker_count: house_info.worker_count(),
            warehouse_count: house_info.warehouse_count(),
            stable_count: house_info.stable_count(),

            craft_list: house_info.craft_list.clone(),
        }
    }
}
#[derive(Deserialize, Debug)]
pub(crate) struct HouseInfos(HashMap<String, HouseInfo>);

#[allow(unused)]
#[derive(Deserialize, Debug)]
pub(crate) struct HouseInfo {
    pub affiliated_town: usize,
    pub character_key: usize,       /* key [CharacterKey] */
    pub craft_list: Vec<CraftList>, /* house craft list [pa_vector<gc::HouseInfoCraft>] */
    pub len_need_house_key: usize,
    pub house_floor: usize,      /* house floor [HouseFloor] */
    pub house_group: usize,      /* house group [HouseGroup] */
    pub need_explore_point: u32, /* need explore count [ExplorationPoint] */
    pub need_house_key: usize,   /* need house key list [pa_vector<gc::CharacterKey>] */
    pub num_craft_list_items: u32,
    pub parent_node: usize, /* parent node key [gc::WaypointKey] */
}

impl HouseInfo {
    pub fn worker_count(&self) -> usize {
        self.craft_index_to_count(1)
    }

    pub fn warehouse_count(&self) -> usize {
        self.craft_index_to_count(2)
    }

    pub fn stable_count(&self) -> usize {
        self.craft_index_to_count(3)
    }

    fn craft_index_to_count(&self, index: usize) -> usize {
        self.craft_list
            .iter()
            .find(|c| c.item_craft_index == index)
            .map(|c| c.level_to_count() as usize)
            .unwrap_or(0)
    }
}

#[derive(Clone, Deserialize, Debug)]
pub(crate) struct CraftList {
    pub house_level: u32,
    pub item_craft_index: usize,
}

impl CraftList {
    pub fn level_to_count(&self) -> u32 {
        if self.item_craft_index == 0 || self.item_craft_index > 3 {
            return 0;
        }

        match self.item_craft_index {
            1 => match self.house_level {
                1 => 1,
                2 => 2,
                3 => 4,
                4 => 6,
                5 => 8,
                _ => 0,
            },
            2 => match self.house_level {
                1 => 3,
                2 => 5,
                3 => 8,
                4 => 12,
                5 => 16,
                _ => 0,
            },
            3 => match self.house_level {
                1 => 1,
                2 => 2,
                3 => 3,
                4 => 4,
                5 => 5,
                _ => 0,
            },
            _ => 0,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct LanguageData {
    param0: usize,
    string: String,
}

pub(crate) fn read_csv_data(filename: &str) -> Result<IndexedStringMap> {
    let path = Path::new("./data/houseinfo/filename.txt");
    let path = path.with_file_name(filename);
    let path_string = path.clone();
    let path_string = path_string.as_path().display();
    let file = std::fs::File::open(path).context(format!("Can't find {path_string})"))?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut records: std::collections::HashMap<usize, String> = std::collections::HashMap::new();
    for result in rdr.deserialize() {
        let record: LanguageData = result?;
        records.insert(record.param0, record.string);
    }
    Ok(records)
}

pub(crate) fn parse_houseinfo_data() -> Result<RegionBuildingMap> {
    let character = read_csv_data("Character.csv")?;
    let exploration = read_csv_data("Exploration.csv")?;
    let region = read_csv_data("Region.csv")?;

    let json_string = std::fs::read_to_string("./data/houseinfo/houseinfo.json")?;
    let house_infos_map: HashMap<String, HouseInfo> = serde_json::from_str(&json_string)?;
    let house_infos: Vec<HouseInfo> = house_infos_map.into_values().collect();

    let mut region_buildings = RegionBuildingMap::new();

    for house_info in house_infos.into_iter() {
        let building = Building::new(&house_info, &character, &exploration, &region);
        if let Some(region) = region_buildings.get_mut(&building.region_name) {
            region.insert(building.key, building);
        } else {
            let region_key = building.region_name.to_owned();
            let mut value = BuildingMap::new();
            value.insert(building.key, building);
            region_buildings.insert(region_key, value);
        }
    }
    Ok(region_buildings)
}

pub(crate) fn get_region_buildings(region_name: Option<String>) -> Result<RegionBuildingMap> {
    let mut region_buildings = parse_houseinfo_data()?;
    if let Some(region_name) = region_name {
        region_buildings.retain(|k, _| *k == region_name);
        if region_buildings.len() != 1 {
            bail!(
                "Unable to find region {}.\n Use '--list-regions'.",
                region_name
            );
        }
    };
    Ok(region_buildings)
}
