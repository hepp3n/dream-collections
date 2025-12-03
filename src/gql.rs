use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct Vars {
    pub filter: Filter,
    pub limit: u32,
    pub offset: u32,
    pub sort: Sort,
}

#[derive(Debug, Serialize)]
pub struct Filter {
    pub dd: Option<Vec<u8>>,
    pub dsr: Option<Vec<u8>>,
    pub iml: Option<Vec<u8>>,
    pub imsd: Option<Vec<u8>>,
    pub izdr: Option<Vec<u8>>,
    pub rd: Option<Vec<u8>>,
    #[serde(rename = "type")]
    pub item_type: Option<Vec<String>>,
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Sort {
    pub field: String,
    #[serde(rename = "type")]
    pub sort_type: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Data {
    pub lots: Lots,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Lots {
    #[serde(rename = "Lots")]
    pub lots: Vec<Item>,
    #[serde(rename = "Pagination")]
    pub pagination: Pagination,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: Option<String>,
    pub source: Option<String>,
    pub is_mine: Option<bool>,
    #[serde(rename = "type")]
    pub item_type: Option<String>,
    pub gear_score: Option<u32>,
    pub has_pending_counter_offer: Option<bool>,
    #[serde(rename = "Prices")]
    pub prices: Vec<Prices>,
    #[serde(rename = "Currencies")]
    pub currencies: Option<Vec<Currencies>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Prices {
    pub value: Option<f64>,
    #[serde(rename = "Currency")]
    pub currency: Currency,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Currency {
    pub id: Option<u8>,
    pub code: Option<String>,
    #[serde(rename = "type")]
    pub currency_type: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Currencies {
    pub id: Option<u8>,
    pub code: Option<String>,
    #[serde(rename = "type")]
    pub currencies_type: Option<String>,
    pub title: Option<String>,
    pub is_available_for_lots: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Pagination {
    pub total: u32,
    pub current_page: u32,
    pub next_page_exists: bool,
}
