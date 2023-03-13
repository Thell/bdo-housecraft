use anyhow::{Context, Ok, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::path::Path;

pub(crate) type IndexedStringMap = HashMap<u32, String>;
pub(crate) type BuildingMap = BTreeMap<u32, Building>;
pub(crate) type RegionBuildingMap = BTreeMap<String, BuildingMap>;
pub(crate) type CraftBuildingMap = BTreeMap<String, (u32, Vec<Building>)>;

// pub const HOUSECRAFT_TABLE_STYLE: &str = "0123456789abcdefghi";
pub(crate) const HOUSECRAFT_TABLE_STYLE: &str = "   ═────      ═  ══";

#[allow(unused)]
#[derive(Clone, Debug)]
pub(crate) struct Building {
    pub key: u32,
    pub need_key: u32,
    pub node_key: u32,
    pub region_key: u32,
    pub building_name: String,
    pub node_name: String,
    pub region_name: String,
    pub cost: u32,
    pub worker_count: u32,
    pub stable_count: u32,
    pub warehouse_count: u32,
    pub craft_list: Vec<CraftList>,
}

impl Building {
    pub fn new(
        house_info: &HouseInfo,
        character: &IndexedStringMap,
        exploration: &IndexedStringMap,
        region: &IndexedStringMap,
        _usage: &IndexedStringMap,
    ) -> Self {
        let key = house_info.character_key;
        let node_key = house_info.parent_node;
        let region_key = house_info.affiliated_warehouse;

        Self {
            key,
            need_key: if house_info.has_need_house_key == 1 {
                house_info.need_house_key
            } else {
                house_info.affiliated_warehouse
            },
            node_key,
            region_key,

            building_name: character.get(&key).unwrap().to_string(),
            node_name: exploration.get(&node_key).unwrap().to_string(),
            region_name: region.get(&region_key).unwrap().to_string(),

            cost: house_info.need_explore_point,
            worker_count: house_info.worker_count(),
            warehouse_count: house_info.warehouse_count(),
            stable_count: house_info.stable_count(),

            craft_list: house_info.craft_list.clone(),
        }
    }
}
#[derive(Deserialize, Debug)]
pub(crate) struct HouseInfos {
    pub house_info: Vec<HouseInfo>,
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
pub(crate) struct HouseInfo {
    pub affiliated_warehouse: u32,
    pub character_key: u32,         /* key [CharacterKey] */
    pub craft_list: Vec<CraftList>, /* house craft list [pa_vector<gc::HouseInfoCraft>] */
    pub has_need_house_key: u32,
    pub house_floor: u32,        /* house floor [HouseFloor] */
    pub house_group: u32,        /* house group [HouseGroup] */
    pub need_explore_point: u32, /* need explore count [ExplorationPoint] */
    pub need_house_key: u32,     /* need house key list [pa_vector<gc::CharacterKey>] */
    pub num_craft_list_items: u32,
    pub parent_node: u32, /* parent node key [gc::WaypointKey] */
}

impl HouseInfo {
    pub fn worker_count(&self) -> u32 {
        self.craft_index_to_count(1)
    }

    pub fn warehouse_count(&self) -> u32 {
        self.craft_index_to_count(2)
    }

    pub fn stable_count(&self) -> u32 {
        self.craft_index_to_count(3)
    }

    fn craft_index_to_count(&self, index: u32) -> u32 {
        self.craft_list
            .iter()
            .find(|c| c.item_craft_index == index)
            .map(|c| c.level_to_count())
            .unwrap_or(0)
    }
}

#[derive(Clone, Deserialize, Debug)]
pub(crate) struct CraftList {
    pub house_level: u32,
    pub item_craft_index: u32,
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
    param0: u32,
    string: String,
}

pub(crate) fn read_csv_data(filename: &str) -> Result<HashMap<u32, String>> {
    let path = Path::new("./data/houseinfo/filename.txt");
    let path = path.with_file_name(filename);
    let path_string = path.clone();
    let path_string = path_string.as_path().display();
    let file = std::fs::File::open(path).context(format!("Can't find {path_string})"))?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut records: std::collections::HashMap<u32, String> = std::collections::HashMap::new();
    for result in rdr.deserialize() {
        let record: LanguageData = result?;
        records.insert(record.param0, record.string);
    }
    Ok(records)
}

pub(crate) fn parse_houseinfo_data() -> Result<RegionBuildingMap> {
    let character = read_csv_data("Character.csv")?;
    let exploration = read_csv_data("Exploration.csv")?;
    let usage = read_csv_data("HouseInfoReceipe.csv")?;
    let region = read_csv_data("RegionInfo.csv")?;

    let json_string = std::fs::read_to_string("./data/houseinfo/houseinfo.json")?;
    let house_infos = serde_json::from_str::<HouseInfos>(&json_string)?;

    let mut region_buildings = RegionBuildingMap::new();

    for house_info in house_infos.house_info.into_iter() {
        let building = Building::new(&house_info, &character, &exploration, &region, &usage);
        if let Some(region) = region_buildings.get_mut(&building.region_name) {
            region.insert(building.key, building);
        } else {
            let region_key = building.region_name.to_owned();
            let mut value = BTreeMap::<u32, Building>::new();
            value.insert(building.key, building);
            region_buildings.insert(region_key, value);
        }
    }
    Ok(region_buildings)
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// I think I'll use these during generate...

// pub(crate) fn town_id_to_name(key: u32) -> String {
//     match key {
//         5 => "Velia".to_string(),
//         32 => "Heidel".to_string(),
//         52 => "Glish".to_string(),
//         77 => "Calpheon City".to_string(),
//         88 => "Olvia".to_string(),
//         107 => "Keplan".to_string(),
//         120 => "Port Epheria".to_string(),
//         126 => "Trent".to_string(),
//         182 => "Iliya Island".to_string(),
//         202 => "Altinova".to_string(),
//         221 => "Tarif".to_string(),
//         229 => "Valencia City".to_string(),
//         601 => "Shakatu".to_string(),
//         605 => "Sand Grain Bazaar".to_string(),
//         619 => "Ancado Inner Harbor".to_string(),
//         693 => "Arehaza".to_string(),
//         694 => "Muiquun".to_string(),
//         706 => "Old Wisdom Tree".to_string(),
//         735 => "Grána".to_string(),
//         873 => "Duvencrune".to_string(),
//         955 => "O'draxxia".to_string(),
//         1124 => "Eilton".to_string(),
//         _ => {
//             panic!("Invalid Town key.")
//         }
//     }
// }

// pub(crate) fn get_town_buildings_by_name(town_name: &str) -> Result<BuildingMap> {
//     let regions_buildings = merge_houseinfo_data()?;
//     let region_buildings = regions_buildings
//         .get(town_name)
//         .context(format!("Error getting {town_name} buildings."))?;
//     Ok(region_buildings.clone())
// }

// pub(crate) fn town_name_to_key(name: &str) -> u32 {
//     match name {
//         "Velia" => 5,
//         "Heidel" => 32,
//         "Glish" => 52,
//         "Calpheon City" => 77,
//         "Olvia" => 88,
//         "Keplan" => 107,
//         "Port Epheria" => 120,
//         "Trent" => 126,
//         "Iliya Island" => 182,
//         "Altinova" => 202,
//         "Tarif" => 221,
//         "Valencia City" => 229,
//         "Shakatu" => 601,
//         "Sand Grain Bazaar" => 605,
//         "Ancado Inner Harbor" => 619,
//         "Arehaza" => 693,
//         "Muiquun" => 694,
//         "Old Wisdom Tree" => 706,
//         "Grána" => 735,
//         "Duvencrune" => 873,
//         "O'draxxia" => 955,
//         "Eilton" => 1124,
//         _ => 0,
//     }
// }

// pub(crate) fn get_town_buildings_by_key(key: u32) -> Result<BuildingMap> {
//     let town_name = town_id_to_name(key);
//     get_town_buildings_by_name(&town_name)
// }

// pub(crate) fn get_town_buildings_hierarchy(
//     town_name: &str,
// ) -> Result<(
//     std::collections::BTreeMap<u32, Building>,
//     usize,
//     Vec<usize>,
//     Vec<usize>,
// )> {
//     let town_key = town_name_to_key(town_name);
//     let buildings = get_town_buildings_by_name(town_name)?;
//     let root = town_key as usize;
//     let mut parents: Vec<usize> = Vec::with_capacity(buildings.len());
//     let mut children: Vec<usize> = Vec::with_capacity(buildings.len());
//     parents.push(usize::MAX);
//     children.push(root);
//     for (k, v) in buildings.iter() {
//         parents.push(v.need_key as usize);
//         children.push(*k as usize);
//     }
//     Ok((buildings, root, parents, children))
// }
