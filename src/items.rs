use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

use crate::gql::Vars;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum ItemType {
    #[default]
    Helm,
    Armor,
    Pants,
    Gloves,
    Boots,
}

impl From<String> for ItemType {
    fn from(item_str: String) -> Self {
        match item_str.as_str() {
            "Helm" => ItemType::Helm,
            "Armor" => ItemType::Armor,
            "Pants" => ItemType::Pants,
            "Gloves" => ItemType::Gloves,
            "Boots" => ItemType::Boots,

            _ => panic!("Unknown item type: {}", item_str),
        }
    }
}

impl Display for ItemType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let item_str = match self {
            ItemType::Helm => "Helm",
            ItemType::Armor => "Armor",
            ItemType::Pants => "Pants",
            ItemType::Gloves => "Gloves",
            ItemType::Boots => "Boots",
        };
        write!(f, "{}", item_str)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ItemOptionType {
    MH = 0,
    SD = 1,
    DD = 2,
    Ref = 3,
    Dsr = 4,
    Zen = 5,
}

impl Display for ItemOptionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let option_str = match self {
            ItemOptionType::MH => "Maximum Life (MH)",
            ItemOptionType::SD => "Increase Maximum SD (SD)",
            ItemOptionType::DD => "Damage Decrease (DD)",
            ItemOptionType::Ref => "Damage Reflection (REF)",
            ItemOptionType::Dsr => "Defense Success Rate (DSR)",
            ItemOptionType::Zen => "Additional Zen drop rate (ZEN)",
        };
        write!(f, "{}", option_str)
    }
}

pub type ItemOption = ItemOptionType;
pub type ItemHasOption = bool;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ItemOptions(pub BTreeMap<ItemOption, ItemHasOption>);

impl Default for ItemOptions {
    fn default() -> Self {
        let mut options = BTreeMap::new();

        options.insert(ItemOptionType::MH, false);
        options.insert(ItemOptionType::SD, false);
        options.insert(ItemOptionType::DD, false);
        options.insert(ItemOptionType::Ref, false);
        options.insert(ItemOptionType::Dsr, false);
        options.insert(ItemOptionType::Zen, false);

        ItemOptions(options)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    pub options: Arc<Mutex<ItemOptions>>,
    pub item_type: Option<ItemType>,
    pub name: Option<String>,
}

impl Item {
    pub fn new(name: String, item_type: ItemType) -> Self {
        Item {
            options: Arc::new(Mutex::new(ItemOptions::default())),
            item_type: Some(item_type),
            name: Some(name),
        }
    }
}

impl Item {
    pub fn generate_market_query(&self) -> String {
        r#"
            query GET_ALL_LOTS($offset: NonNegativeInt, $limit: NonNegativeInt, $sort: LotsSortInput, $filter: LotsFilterInput) {
              lots(limit: $limit, offset: $offset, sort: $sort, filter: $filter) {
                Lots {
                  id
                  source
                  isMine
                  type
                  gearScore
                  hasPendingCounterOffer
                  Prices {
                    value
                    Currency {
                      id
                      code
                      type
                      title
                      __typename
                    }
                    __typename
                  }
                  Currencies {
                    id
                    code
                    type
                    title
                    isAvailableForLots
                    __typename
                  }
                  __typename
                }
                Pagination {
                  total
                  currentPage
                  nextPageExists
                  __typename
                }
                __typename
              }
            }
        "#.to_string()
    }

    pub fn generate_gql_vars(&self) -> Vars {
        let options = self.options.lock().unwrap();

        Vars {
            filter: crate::gql::Filter {
                dd: options.0.get(&ItemOptionType::DD).and_then(|has_option| {
                    if *has_option {
                        Some(vec![0, 1, 2, 3, 4])
                    } else {
                        None
                    }
                }),
                dsr: options.0.get(&ItemOptionType::Dsr).and_then(|has_option| {
                    if *has_option {
                        Some(vec![0, 1, 2, 3, 4])
                    } else {
                        None
                    }
                }),
                iml: options.0.get(&ItemOptionType::MH).and_then(|has_option| {
                    if *has_option {
                        Some(vec![0, 1, 2, 3, 4])
                    } else {
                        None
                    }
                }),
                imsd: options.0.get(&ItemOptionType::SD).and_then(|has_option| {
                    if *has_option {
                        Some(vec![0, 1, 2, 3, 4])
                    } else {
                        None
                    }
                }),
                rd: options.0.get(&ItemOptionType::Ref).and_then(|has_option| {
                    if *has_option {
                        Some(vec![0, 1, 2, 3, 4])
                    } else {
                        None
                    }
                }),
                izdr: options.0.get(&ItemOptionType::Zen).and_then(|has_option| {
                    if *has_option {
                        Some(vec![0, 1, 2, 3, 4])
                    } else {
                        None
                    }
                }),

                item_type: Some(vec![
                    self.item_type.as_ref().unwrap().to_string().to_lowercase(),
                ]),
                name: self.name.clone(),
            },
            limit: 200,
            offset: 0,
            sort: crate::gql::Sort {
                field: "LOT_FIELD_MIN_PRICE".to_string(),
                sort_type: "SORT_TYPE_ASC".to_string(),
            },
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SetItems {
    pub set_string: String,
    pub set: AllSets,
    pub items: [Arc<Mutex<Item>>; 5],
}

impl From<String> for SetItems {
    fn from(set_str: String) -> Self {
        let set_name: AllSets = AllSets::from(set_str.clone());
        SetItems::new(set_name)
    }
}

impl Display for SetItems {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.set_string)
    }
}

impl SetItems {
    pub fn new(set_name: AllSets) -> Self {
        SetItems {
            set_string: format!("{}", set_name),
            set: set_name.clone(),
            items: [
                Arc::new(Mutex::new(Item::new(
                    format!("{}", set_name),
                    ItemType::Helm,
                ))),
                Arc::new(Mutex::new(Item::new(
                    format!("{}", set_name),
                    ItemType::Armor,
                ))),
                Arc::new(Mutex::new(Item::new(
                    format!("{}", set_name),
                    ItemType::Pants,
                ))),
                Arc::new(Mutex::new(Item::new(
                    format!("{}", set_name),
                    ItemType::Gloves,
                ))),
                Arc::new(Mutex::new(Item::new(
                    format!("{}", set_name),
                    ItemType::Boots,
                ))),
            ],
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ClassSets {
    DarkWizard(Vec<SetItems>),
    DarkKnight(Vec<SetItems>),
    Elf(Vec<SetItems>),
    MagicGladiator(Vec<SetItems>),
    DarkLord(Vec<SetItems>),
    Summoner(Vec<SetItems>),
    RageFighter(Vec<SetItems>),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum AllSets {
    // Dark Wizard Sets
    Pad,
    Bone,
    Sphinx,
    Legendary,
    GrandSoul,
    DarkSoul,
    VenomMist,
    // Dark Knight Sets
    Leather,
    Bronze,
    Scale,
    Brass,
    Plate,
    Dragon,
    BlackDragon,
    DarkPhoenix,
    GreatDragon,
    DragonKnight,
    // Elf Sets
    Vine,
    Silk,
    Wind,
    Spirit,
    Guardian,
    HolySpirit,
    RedSpirit,
    SylphidRay,
    // Magic Gladiator Sets
    StormCrow,
    ThunderHawk,
    Hurricane,
    Volcano,
    // Dark Lord Sets
    LightPlate,
    Adamantine,
    DarkSteel,
    DarkMaster,
    Sunlight,
    // Summoner Sets
    ViolentWind,
    RedWinged,
    Ancient,
    Demonic,
    StormBlitz,
    Succubus,
    // Rage Fighter Sets
    SacredFire,
    StormZahard,
    PiercingGrove,
    PhoenixSoul,
}

impl Display for AllSets {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let set_str = match self {
            AllSets::Pad => "Pad",
            AllSets::Bone => "Bone",
            AllSets::Sphinx => "Sphinx",
            AllSets::Legendary => "Legendary",
            AllSets::GrandSoul => "Grand Soul",
            AllSets::DarkSoul => "Dark Soul",
            AllSets::VenomMist => "Venom Mist",
            AllSets::Leather => "Leather",
            AllSets::Bronze => "Bronze",
            AllSets::Scale => "Scale",
            AllSets::Brass => "Brass",
            AllSets::Plate => "Plate",
            AllSets::Dragon => "Dragon",
            AllSets::BlackDragon => "Black Dragon",
            AllSets::DarkPhoenix => "Dark Phoenix",
            AllSets::GreatDragon => "Great Dragon",
            AllSets::DragonKnight => "Dragon Knight",
            AllSets::Vine => "Vine",
            AllSets::Silk => "Silk",
            AllSets::Wind => "Wind",
            AllSets::Spirit => "Spirit",
            AllSets::Guardian => "Guardian",
            AllSets::HolySpirit => "Holy Spirit",
            AllSets::RedSpirit => "Red Spirit",
            AllSets::SylphidRay => "Sylphid Ray",
            AllSets::StormCrow => "Storm Crow",
            AllSets::ThunderHawk => "Thunder Hawk",
            AllSets::Hurricane => "Hurricane",
            AllSets::Volcano => "Volcano",
            AllSets::LightPlate => "Light Plate",
            AllSets::Adamantine => "Adamantine",
            AllSets::DarkSteel => "Dark Steel",
            AllSets::DarkMaster => "Dark Master",
            AllSets::Sunlight => "Sunlight",
            AllSets::ViolentWind => "Violent Wind",
            AllSets::RedWinged => "Red Winged",
            AllSets::Ancient => "Ancient",
            AllSets::Demonic => "Demonic",
            AllSets::StormBlitz => "Storm Blitz",
            AllSets::Succubus => "Succubus",
            AllSets::SacredFire => "Sacred Fire",
            AllSets::StormZahard => "Storm Zahard",
            AllSets::PiercingGrove => "Piercing Grove",
            AllSets::PhoenixSoul => "Phoenix Soul",
        };
        write!(f, "{}", set_str)
    }
}

impl From<String> for AllSets {
    fn from(set_str: String) -> Self {
        match set_str.as_str() {
            // Dark Wizard Sets
            "Pad" => AllSets::Pad,
            "Bone" => AllSets::Bone,
            "Sphinx" => AllSets::Sphinx,
            "Legendary" => AllSets::Legendary,
            "Grand Soul" => AllSets::GrandSoul,
            "Dark Soul" => AllSets::DarkSoul,
            "Venom Mist" => AllSets::VenomMist,
            // Dark Knight Sets
            "Leather" => AllSets::Leather,
            "Bronze" => AllSets::Bronze,
            "Scale" => AllSets::Scale,
            "Brass" => AllSets::Brass,
            "Plate" => AllSets::Plate,
            "Dragon" => AllSets::Dragon,
            "Black Dragon" => AllSets::BlackDragon,
            "Dark Phoenix" => AllSets::DarkPhoenix,
            "Great Dragon" => AllSets::GreatDragon,
            "Dragon Knight" => AllSets::DragonKnight,
            // Elf Sets
            "Vine" => AllSets::Vine,
            "Silk" => AllSets::Silk,
            "Wind" => AllSets::Wind,
            "Spirit" => AllSets::Spirit,
            "Guardian" => AllSets::Guardian,
            "Holy Spirit" => AllSets::HolySpirit,
            "Red Spirit" => AllSets::RedSpirit,
            "Sylphid Ray" => AllSets::SylphidRay,
            // Magic Gladiator Sets
            "Storm Crow" => AllSets::StormCrow,
            "Thunder Hawk" => AllSets::ThunderHawk,
            "Hurricane" => AllSets::Hurricane,
            "Volcano" => AllSets::Volcano,
            // Dark Lord Sets
            "Light Plate" => AllSets::LightPlate,
            "Adamantine" => AllSets::Adamantine,
            "Dark Steel" => AllSets::DarkSteel,
            "Dark Master" => AllSets::DarkMaster,
            "Sunlight" => AllSets::Sunlight,
            // Summoner Sets
            "Violent Wind" => AllSets::ViolentWind,
            "Red Winged" => AllSets::RedWinged,
            "Ancient" => AllSets::Ancient,
            "Demonic" => AllSets::Demonic,
            "Storm Blitz" => AllSets::StormBlitz,
            "Succubus" => AllSets::Succubus,
            // Rage Fighter Sets
            "Sacred Fire" => AllSets::SacredFire,
            "Storm Zahard" => AllSets::StormZahard,
            "Piercing Grove" => AllSets::PiercingGrove,
            "Phoenix Soul" => AllSets::PhoenixSoul,
            _ => panic!("Unknown set name: {}", set_str),
        }
    }
}
